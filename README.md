# Nasir

A goofy lil cli web browser written in rust

## How to use

`cargo run --release -- https://www.wikipedia.org`

The `--release` flag is very important for performance. Some urls need to be enclosed in "quotes" or cargo will complain

### Controls

The location of your cursor is determined by the `>` on the left side of the screen. Many of Nasir's controls are based on Vim.

The up and down arrow keys, and `j` and `k` move your cursor up and down. Page up and page down will move the cursor by 10 lines at a time. The window will automatically scroll to keep your cursor in view.

To follow a link, use `enter`. Some links, like those that start with `#`, don't work yet.

To directly navigate to a web address, type `:` and then type the address and press enter.

To go back to the previously visited page, or exit the program if you're on the first page visited, use the `esc` key.

To reload the page, use the `r` key. This can be used to load images.

To copy the text on the current line, use the `y` key.

### Document Elements

Each line of the terminal is a text element on the screen. Nasir only specially renders a few types of rich text elements.

Links are rendered underlined in blue. When selected, they show up in the format of a `[markdown](link)`, with the destination address underlined in blue.

Within html pages, images are lazily rendered. Before rendering, it will show as `[image]`, `[image: alt text]`, or `[image: path/to/file]`. The next time the page is loaded after the image data is received, it will be replaced with a pixelated approximation of the image half the height and half the width of the screen. If the image is in an unsupported format, it will never be replaced.

If you navigate to the link directly to an image, it will take up the whole screen. If you make the terminal smaller or increase the text size, the image will break until you reload.

### Compatability

Nasir is compatible with very few websites. For example, [YouTube](https://www.youtube.com) has a `<` character in its javascript that's delivered on initial page load. This causes the HTML parser to enter a loop, which can only be stopped via task manager. [Twitter](https://www.twitter.com) has too many redirects for reqwest to process. [Facebook](https://www.facebook.com) complains that Nasir isn't supported by Facebook. [The Rust Foundation](https://foundtion.rust-lang.org) divides by zero when you try to reload the page. Finally, anything that uses javascript will not work.
