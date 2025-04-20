To create a new favicon:

```bash
convert input.png -resize 16x16 favicon-16.png
convert input.png -resize 32x32 favicon-32.png
convert input.png -resize 48x48 favicon-48.png

# Combine them into a single .ico
convert favicon-16.png favicon-32.png favicon-48.png favicon.ico
```
