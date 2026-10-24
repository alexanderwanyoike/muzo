# Bundled asset notices

Manrope Variable is bundled into the desktop renderer. Its SIL Open Font License
is retained in `manrope-OFL.txt`, copied unchanged from
`@fontsource-variable/manrope/LICENSE`. The assets directory is included in the
Forge package and in every installer made from that package.

Electron's own LICENSE and Chromium notices remain with the original packaged
runtime. The installer stage uses that complete Forge payload without stripping
notices or rewriting the application profile and credential identity.
