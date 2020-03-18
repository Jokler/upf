# upf
upf allows the sending of requests based on template files.
These template files were inspired by custom uploaders in ShareX.

This could be used to share images on imgur if the appropriate
template was stored under `~/.config/upf/imgur.toml` for example:
```sh
maim -s /tmp/screenshot.png && upf imgur /tmp/screenshot.png
```
