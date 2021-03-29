# term-image

Display images and gifs in high resolution in your terminal.

The best way to use 80s technology to view pictures and gifs!

## About

term-image lets you put images in previously text only locations (like terminals)
by converting images to characters with foreground and background colors.

term-image supports several methods for rendering images, including:

* Unicode half blocks, fractional blocks, and drawing characters
* Unicode braille characters
* ASCII text
* iTerm2 and kitty protocols to render images in full resolutions

Supports both truecolor (16M RGB color) and ANSI Color (256 color).

For transparent images, a background color can be provided (such as a terminal background color) which will be used
to remove the alpha from the image for processing.

term-image is implemented as a freestanding library and designed to be integrated into other applications.

# Examples 
Truecolor, full unicode 

<img src="https://i.imgur.com/6DFX97t.png" alt="Lichtenstein" width="50%"/>

Truecolor, full unicode  (small font size)

<img src="https://i.imgur.com/hOMUaj2.png" alt="Big parrot" width="50%"/>

See [term-image-web](https://noskcaj19.github.io/term-image-web/) for a live web demo.

Gif, truecolor, unicode blocks
![gif](https://i.imgur.com/UNputum.png)

Gif, truecolor, ascii
[![asciicast](https://asciinema.org/a/190320.png)](https://asciinema.org/a/190320)
(you may need to replay the asciicast to get smooth playback)

## How it works

### Block
The block renderer uses a mapping of bitmap to a unicode drawing character (["Box Drawing"](https://en.wikipedia.org/wiki/Box_Drawing_(Unicode_block)) and
["Block Elements"](https://en.wikipedia.org/wiki/Block_Elements#Compact_table)) to find a character that most closely
matches the "shape" of each 4x8 pixel block.

Each of these bitmaps represent the "dark" section of each character.

For example, given the character `┫`, the associated bitmap is `0x666ee666`, which is a 1d representation of the 2d
bitmap:
```code
.##.
.##.
.##.
###.
###.
.##.
.##.
.##.
```
where "#" represents 1 and "." represents 0.

### Braille
[Unicode Braille Patterns](https://en.wikipedia.org/wiki/Braille_Patterns) allow for 1 to 1 resolution images
but each 2x4 rectangle can only have a single foreground and background color, which makes color representation
subpar.

The braille renderer converts the input image to greyscale and maps 2x4 pixel blocks directly to a braille pattern.
Then the foreground color is determined by the same technique as the block renderer.

### ASCII
The ASCII renderer simply converts the input image to greyscale and finds the ascii character
with the closest matching character.

The mapping from character to brightness was created by a script that rendered each character and counted
the number of black pixels.

It currently works alright, but has lots of room for improvement, for example see libcaca. (I plan to add
libcaca support eventually)

### iTerm and Kitty
These "proprietary" renders use the "proprietary" protocols defined by 
[iTerm2](https://iterm2.com/documentation-images.html) and [kitty](https://sw.kovidgoyal.net/kitty/graphics-protocol.html)
and as such will only function in terminals that support the respective protocol. (For example, 
[wezterm](https://wezfurlong.org/wezterm/) implements the [iterm image protocol](https://wezfurlong.org/wezterm/imgcat.html)
and can therefore use the iterm renderer).