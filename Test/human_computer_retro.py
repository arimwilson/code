# HumanComputerRetro - making old software fun for computers AND humans
#
# Stages:
# 1) Determine video parameters based on input file and options.
# 2) Turn binary into blocks of color in rgb24 frames using NumPy
# 3) Use ffmpeg via pipes to encode for YouTube
import argparse, io, math, numpy, os, subprocess, time
from PIL import Image

class VideoParameters:
  def __init__(
      self, length_seconds, height, width, color_palette, bytes_per_second):
    self.length_seconds = length_seconds
    self.height = height
    self.width = width
    self.color_palette = color_palette
    self.bytes_per_second = bytes_per_second

  # Approximate max data/s from
  # https://support.google.com/youtube/answer/1722171?hl=en#zippy=%2Cbitrate:
  #   720p60: 937.5 kilobyte/s
  #   1080p60: 1500 kilobyte/s
  #   4k60: 6625 kilobyte/s
  @classmethod
  def get(cls, file_size_bytes, length_seconds, color_palette, resolution):
    if resolution == '4k':
      width = 3840
      height = 2160
      max_bytes_per_second = 6625000
    elif not resolution or resolution == '1080p':
      width = 1920
      height = 1080
      max_bytes_per_second = 1500000
    elif resolution == '720p':
      width = 1280
      height = 720
      max_bytes_per_second = 937500
    else:
      raise ValueError('invalid resolution')
    bytes_per_second = int(file_size_bytes / length_seconds)
    if bytes_per_second > max_bytes_per_second:
      min_length_seconds = math.ceil(
          file_size_bytes / max_bytes_per_second)
      raise ValueError(
          'file size, output resolution, and desired video length are '
          'incompatible. video length of ' + str(min_length_seconds) +
          ' would work.')
    return cls(length_seconds, height, width, color_palette, bytes_per_second)

  def __repr__(self):
    lst = [str(self.length_seconds), str(self.height), str(self.width),
           self.color_palette, str(self.bytes_per_second)]
    return 'VideoParameters(' + ', '.join(lst) + ')'

def read_in_chunks(file_object, chunk_size=1024):
    while True:
      data = file_object.read(chunk_size)
      if not data:
        break
      yield data

def data_in_chunks(lst, n):
    for i in range(0, len(lst), n):
        yield lst[i:i + n]

# Repeat each element for h rows and w columns
def repeat_elements(a, h, w):
    b = numpy.repeat(a, h, axis=0)
    return numpy.repeat(b, w, axis=1)

# Solve integer linear program to determine block size in pixels using brute
# force
def get_block_size_in_pixels(params, min_bytes_in_frame):
  for block_size in range(1, min(params.height, params.width)):
    if int(params.width / block_size) * int(params.height / block_size) <\
        min_bytes_in_frame / 3:
      return block_size - 1

# map data to a color in color palette
def lookup_color(color_palette, frame_pixel):
  return color_palette[int(numpy.bitwise_xor.reduce(frame_pixel))]

# Human pattern recogition takes about 166ms, so we generate 6 independent
# frames at 60fps to get 1 second of footage. Bytes visualized should be >=
# bytes_per_second. Each frame is made up of blocks of color.
def generate_frames(params, data):
  min_bytes_in_frame = int(params.bytes_per_second / 6) + 1
  block_size = get_block_size_in_pixels(params, min_bytes_in_frame)
  row_blocks = int(params.height / block_size)
  column_blocks = int(params.width / block_size)
  frames = []
  for frame_data in data_in_chunks(data, row_blocks * column_blocks * 3):
    frame = numpy.frombuffer(frame_data, dtype=numpy.uint8)
    # resize to rectangular set of pixels
    frame = numpy.reshape(
        numpy.pad(frame, (0, row_blocks * column_blocks * 3 - frame.size)),
        (row_blocks, column_blocks, 3))
    if params.color_palette is not None:
      # map every pixel to some color in palette
      frame = numpy.apply_along_axis(
          lambda pixel: lookup_color(params.color_palette, pixel), -1, frame)
    # duplicate pixels to create color blocks
    frame = repeat_elements(frame, block_size, block_size)
    # pad out missing space for full frame
    frame = numpy.pad(
        frame,
        ((0, params.height - frame.shape[0]),
         (0, params.width - frame.shape[1]),
         (0, 0)))
    for i in range(10):
      frames.append(frame)
    if len(frames) % 60 == 0:
      yield frames
      frames = []

def get_image_palette(image):
  """
  Return palette in descending order of frequency
  """
  arr = numpy.asarray(image)
  palette, index = numpy.unique(asvoid(arr).ravel(), return_inverse=True)
  palette = palette.view(arr.dtype).reshape(-1, arr.shape[-1])
  count = numpy.bincount(index)
  order = numpy.argsort(count)
  palette = palette[order[::-1]][:256]
  # palette = ','.join(
  #    [''.join([int(rgb).to_bytes(1, 'big').hex() for rgb in color])\ 
  #         for color in palette])
  return palette[:256]

def asvoid(arr):
  """View the array as dtype np.void (bytes)
  This collapses ND-arrays to 1D-arrays, so you can perform 1D operations on them.
  http://stackoverflow.com/a/16216866/190597 (Jaime)
  http://stackoverflow.com/a/16840350/190597 (Jaime)
  Warning:
  >>> asvoid([-0.]) == asvoid([0.])
  array([False], dtype=bool)
  """
  arr = numpy.ascontiguousarray(arr)
  return arr.view(numpy.dtype(
      (numpy.void, arr.dtype.itemsize * arr.shape[-1])))

def get_color_palette(color_palette_text, color_palette_image):
  if color_palette_text is not None and color_palette_image is not None:
    raise ValueError('invalid specification of both forms of color palette')
  elif color_palette_text is not None:
    color_palette = [
        numpy.frombuffer(bytes.fromhex(color), dtype=numpy.uint8) for color\
            in color_palette_text.split(',')]
    # duplicate colors to fill out full 256 color palette
    color_palette_length = len(color_palette)
    i = 0
    while len(color_palette) < 256:
      color_palette.append(color_palette[i])
      i = (i + 1) % color_palette_length
    return color_palette
  elif color_palette_image is not None:
    image = Image.open(color_palette_image, 'r').convert('RGB')
    return get_image_palette(image)
  else:
    return None

def main():
  start = time.time()
  parser = argparse.ArgumentParser(description=
    'Convert binary data relatively losslessly to YouTube-compatible video '
    'that is interesting to humans.')
  parser.add_argument('--input_file', help='Input file.')
  parser.add_argument('--video_length_seconds', type=int, help=
      'Approximate length of video in seconds.')
  parser.add_argument('--color_palette_text', help=
      'Video color palette. Default to full 24-bit color. Specify with up to '
      '256 colors represented as e.g., black, white: 000000,FFFFFF.')
  parser.add_argument('--color_palette_image', help=
      'Video color palette from an image. Will take up to top 256 colors from '
      'image.')
  parser.add_argument('--resolution', help=
      'Video resolution. Defaults to 1080p. Accceptable options are 720p, '
      '1080p, and 4k.')
  parser.add_argument('--music_file', help=
      'Music file to attach to video. Default is no music.')
  parser.add_argument('--ffmpeg', help='Location of ffmpeg.')
  parser.add_argument('--output_video', help='Output video file.')
  args = parser.parse_args()
  input_file_size = os.path.getsize(args.input_file)
  color_palette = get_color_palette(
      args.color_palette_text, args.color_palette_image)
  parameters = VideoParameters.get(
      input_file_size, args.video_length_seconds, color_palette,
      args.resolution)
  command = [
      args.ffmpeg,
      '-y',
      '-f' ,'rawvideo',
      '-s', str(parameters.width)+'x'+str(parameters.height),
      '-pix_fmt', 'rgb24',
      '-r', '60',
      '-i', '-' ]
  if args.music_file is not None:
    command.extend([
      '-i', args.music_file ])
  else:
    command.append('-an')
  command.extend([
    '-c:v', 'libx264',
    '-profile:v', 'high444',
    '-c:a', 'aac',
    args.output_video ])
  pipe = subprocess.Popen(
      command, stdin=subprocess.PIPE, stdout=subprocess.DEVNULL,
      stderr=subprocess.STDOUT)
  frame_count = 0
  with open(args.input_file, 'rb') as input_file:
    for data in read_in_chunks(input_file, 4000000):
      # read enough data for a second of video, convert to frames, output via
      # ffmpeg, then continue.
      for frames in generate_frames(parameters, data):
        for frame in frames:
          try:
            pipe.stdin.write(frame.tobytes())
          except IOError:
            print(pipe.stderr.read())
            return
        frame_count = frame_count + len(frames)
        print(str(frame_count) + " frames written (" +
              str(int(frame_count/60)) + " second(s)). " +
              str(int(time.time() - start)) + " second(s) elapsed.")
  pipe.stdin.close()
  pipe.wait()

if __name__ == "__main__":
  main()
