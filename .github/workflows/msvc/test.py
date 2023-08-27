#!/usr/bin/env python3
import pefile, sys

pe = pefile.PE(sys.argv[1], fast_load=True)
pe.parse_data_directories(directories=[pefile.DIRECTORY_ENTRY["IMAGE_DIRECTORY_ENTRY_RESOURCE"],])
fi_strings = pe.FileInfo[0][0].StringTable[0].entries

# must match version.rc
print(f"version strings: {fi_strings}")
assert fi_strings == {b"CompanyName": b"nabijaczleweli", b"ProductName": b"rust-embed-resource/example/version", b"ProductVersion": b"2.3.0"}
