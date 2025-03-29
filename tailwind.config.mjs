/** @type {import('tailwindcss').Config} */
export default {
  content: ["./src/**/*.{js,ts,jsx,tsx,mdx}"],
  theme: {
    extend: {
      width: {
        "screen-offset": "calc(100vw - 8rem)",
      },
    },
  },
  plugins: [],
};
