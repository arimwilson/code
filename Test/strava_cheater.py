#!/usr/bin/python

# Fiji Way points:
# Start: 33.973542,-118.445449
# End: 33.977137,-118.438389
# Time: 1pm Saturday.
# Height ~5m.

import sys

assert len(sys.argv) == 6

start = (float(sys.argv[1]), float(sys.argv[2]))
end = (float(sys.argv[3]), float(sys.argv[4]))
# Convert start/end to some 2D projection, Pythagoras to get distance, invert to get
# semi-evenly spaced points at desired speed between the two points.
gpx_file = open(sys.argv[5], "w")

gpx_point = """<trkpt lat="%(latitude)f" lon="%(longitude)">
<ele>%(elevation)f</ele>
<time>%(time)s</time>
</trkpt>"""

gpx_template = """<?xml version="1.0" encoding="UTF-8"?>
<gpx
version="1.1"
creator="Created by Ari on A Computer."
xmlns="http://www.topografix.com/GPX/1/1"
xmlns:topografix="http://www.topografix.com/GPX/Private/TopoGrafix/0/1"
xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
xsi:schemaLocation="http://www.topografix.com/GPX/1/1 http://www.topografix.com/GPX/1/1/gpx.xsd http://www.topografix.com/GPX/Private/TopoGrafix/0/1 http://www.topografix.com/GPX/Private/TopoGrafix/0/1/topografix.xsd">
<metadata>
<name><![CDATA[Track]]></name>
<desc><![CDATA[]]></desc>
</metadata>
<trk>
<name><![CDATA[Track]]></name>
<desc><![CDATA[]]></desc>
<type><![CDATA[cycling]]></type>
<extensions><topografix:color>c0c0c0</topografix:color></extensions>
<trkseg>
%s
</trkseg>
</trk>
</gpx>"""
