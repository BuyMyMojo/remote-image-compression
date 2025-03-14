# Remote Image Compression

An API for compressing an image on the server. main use case is using AVIF at it's slowest preset while away from home 

## Features:

- formats:
  - [x] jpeg
  - [x] avif
  - [ ] jpegxl
  - [ ] webp
  - [ ] png (mainly for using `oxipng -o max`)

- Interface:
  - [ ] Image upload
  - [ ] Compress via image URL

- Backend
  - [X] single executable (for the API server, still need the requirements bellow)
  - [ ] docker image
  - [X] Linux support
  - [ ] Windows support
  
  ❔ Mac Support

### Current requirements:

- magick ([ImageMagick](https://archlinux.org/packages/?name=imagemagick))
- cjpeg ([mozjpeg](https://aur.archlinux.org/packages/mozjpeg) or [libjpeg-turbo](https://archlinux.org/packages/extra/x86_64/libjpeg-turbo/))
- avifenc ([libavif](https://archlinux.org/packages/extra/x86_64/libavif/))