module.exports = {
  packagerConfig: {
    asar: false,
    executableName: "muzo",
    name: "Muzo",
    prune: false,
    ignore: [
      /^\/src($|\/)/,
      /^\/electron($|\/)/,
      /^\/node_modules($|\/)/,
      /^\/src-tauri($|\/)/,
      /^\/coverage($|\/)/,
      /^\/dist-electron\/.*\.map$/,
      /^\/.*\.test\.(ts|tsx)$/,
      /^\/forge.config.cjs$/,
      /^\/index.html$/,
      /^\/tsconfig.*$/,
      /^\/vite.config\.(ts|js|d\.ts)$/,
      /^\/eslint.config.js$/,
    ],
  },
  makers: [
    {
      name: "@electron-forge/maker-zip",
      platforms: ["linux"],
    },
    {
      name: "@electron-forge/maker-deb",
      config: {
        options: {
          name: "muzo",
          bin: "muzo",
          description: "Muzo desktop music player",
          maintainer: "Muzo",
          homepage: "https://github.com/alexanderwanyoike/muzo",
        },
      },
    },
  ],
};
