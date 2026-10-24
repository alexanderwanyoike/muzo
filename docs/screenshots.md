# Desktop screenshots

`site/public/images/muzo-player.jpg` is shared by the README and the project website.
It shows the actual React desktop interface at 1440 x 900, using the current
Brushed aluminium dark material with a Cobalt accent.

The capture uses an isolated browser fixture that supplies fictional libraries,
tracks and playback state through the desktop runtime boundary. The four cover
designs are original geometric SVG artwork exported to PNG for the app's artwork
contract. No real audio plays, no Dropbox requests run and no user catalogue,
credentials, local paths or copyrighted album covers appear in the image.
This is a product screenshot with demo data, not a mockup of future features.

`site/public/images/muzo-personalization.jpg` shows the current Settings interface at
1280 x 900 with Porcelain light and a Rose accent. It was captured from the
maintained Settings layout fixture with a fictional library and Dropbox status.

When refreshing these images, use the actual app components and an isolated
profile or fixture. Select a source and start a demo track so the transport and
current album sleeve are visible. Keep source names generic, verify every cover
loads, and capture the viewport without editing the interface or compositing it.
Use the same canonical image in documentation and on the site.

The listening capture includes Card 038's seek-focus correction: a pointer click
at 1:15 keeps the control focused without a blue rectangle. Keyboard focus remains
visible. Files retain the browser capture's native JPEG encoding and dimensions.
The public/ directory follows Vite's unchanged-asset convention, so sharing-image
URLs remain stable after site builds.
