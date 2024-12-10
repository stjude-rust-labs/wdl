module.exports = {
    plugins: [
        require("tailwindcss"),
        require("autoprefixer"),
        require("postcss-import"),
        require("postcss-url"),
        ...(process.env.NODE_ENV === "production" ? [require("cssnano")] : []),
    ],
};
