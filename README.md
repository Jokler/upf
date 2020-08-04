# upf
upf allows the sending of requests based on template files.
These template files were inspired by custom uploaders in ShareX.

This could be used to share images on jokler.xyz if the appropriate
template was stored under `~/.config/upf/templates/jokler-xyz.toml` for example:
```sh
maim -s /tmp/screenshot.png && upf jokler-xyz /tmp/screenshot.png | xsel --clipboard
```
