/** @type {import('next').NextConfig} */
module.exports = {
    webpack: (config, options) => {
      config.experiments = {
        asyncWebAssembly: true,
        layers: true,
      }
      return config;
    },
    reactStrictMode: true
}
