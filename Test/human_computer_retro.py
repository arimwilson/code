# HumanComputerRetro - making old software fun for computers AND humans
#
# stages:
# 1) determine video parameters based on input file and options
# 1) insert some random bytes into the binary file (copyright)
# 2) based on target video length, input binary size, and max data/frame
#    turn binary into rgb24 frames using NumPy.
# 3) use ffmpeg via pipes to encode for YouTube
import argparse, io, math, numpy, os, subprocess

class VideoParameters:
  def __init__(
      self, length_seconds, height, width, color_palette, bytes_per_second):
    self.length_seconds = length_seconds
    self.height = height
    self.width = width
    self.color_palette = color_palette
    self.bytes_per_second = bytes_per_second

  # approximate max data/frame:
  #   720p60: 15.6 kb/frame
  #   1080p60: 25.6 kb/frame
  #   4k60: 113 kb/frame
  @classmethod
  def get(cls, file_size_bytes, length_seconds, color_palette):
    video_bytes_720p = length_seconds * 15600
    video_bytes_1080p = length_seconds * 25600
    video_bytes_4k = length_seconds * 113000
    height = 720
    width = 1280
    bytes_per_second = 15600
    if file_size_bytes > video_bytes_1080p:
      height = 3840
      width = 2160
      bytes_per_second = 113000
    elif file_size_bytes > video_bytes_720p:
      height = 1920
      width = 1080
      bytes_per_second = 25600
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

def generate_frames(parameters, data):
  # Stupid way to do this is to generate 6 independent frames, repeated 10 times
  # (for one total second). Bytes visualized should be bytes_per_second / 6
  frame_data_length = int(parameters.bytes_per_second / 6) + 1
  for frame_data in data_in_chunks(data, frame_data_length):
    # each frame is made up of blocks of color, based on how much data we can
    # max out the frame with
    frames = []
    block_size_in_pixels = int(math.sqrt(
        parameters.height * parameters.width * 3 / frame_data_length))
    frame = numpy.frombuffer(frame_data, dtype=numpy.uint8)
    frame = numpy.repeat(frame, block_size_in_pixels**2)
    print(len(frame))

    #frame = numpy.fromfunction(
    #    generate_frame_pixel, (parameters.height, parameters.width, 3),
    #    dtype=numpy.uint8, parameters=parameters, frame_data=frame_data)
    for i in range(10):
        frames.append(frame)
    yield frames

def generate_frame_pixel(i, j, k, parameters, frame_data):
    print(i)
    if i*j*k >= len(frame_data):
        return 255
    return frame_data[i*j*k]

def main():
  parser = argparse.ArgumentParser(description=
    'Convert binary data relatively losslessly to YouTube-compatible video '
    'that is interesting to humans.')
  parser.add_argument('--input_file', help='Input file.')
  parser.add_argument('--video_length_seconds', type=int, help=
      'Length of video in seconds.')
  parser.add_argument('--color_palette', help=
      'Video color palette. Default to full 24-bit color.')
  parser.add_argument('--ffmpeg', help='Location of ffmpeg.')
  parser.add_argument('--output_video', help='Output video file.')
  args = parser.parse_args()
  parameters = VideoParameters.get(
      os.path.getsize(args.input_file), args.video_length_seconds,
      args.color_palette)
  # read enough data for a second of video, convert to frames, output via
  # ffmpeg, then continue.
  with open(args.input_file, 'rb') as input_file:
    for data in read_in_chunks(input_file, parameters.bytes_per_second):
        for frames in generate_frames(parameters, data):
            output_ffmpeg(frames)

if __name__ == "__main__":
  main()
