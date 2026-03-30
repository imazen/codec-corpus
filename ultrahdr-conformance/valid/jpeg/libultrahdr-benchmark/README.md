# Files for Ultrahdr Benchmark Test

This folder contains media files for benchmark test in libultrahdr.

Here are the instructions for copying these files manually to the device.

```
$ connect the device under test via ADB.
$ chmod a+x copy_media.sh
$ ./copy_media.sh
```

If there are multiple devices connected under adb, add -s serial option to ./copy_media.sh
