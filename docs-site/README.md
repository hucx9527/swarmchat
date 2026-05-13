# SCP Developer Portal

Static developer portal for the Swarm Communication Protocol.

## Local Development

```bash
# Serve with any static file server
python3 -m http.server 8000

# Or with npx
npx serve .
```

## Deploy

The portal is a static HTML site. Deploy to any hosting platform:

- **GitHub Pages**: Push to `gh-pages` branch
- **Vercel/Netlify**: Point to this directory
- **Any CDN**: Upload the contents

## Structure

```
docs-site/
├── index.html       # Main landing page
├── assets/          # CSS, JS, images
└── README.md
```

## Customization

Edit `index.html` to update:
- Hero section text
- SDK/tool listings
- API reference table
- Architecture diagram
- Footer links

## Related

- [SCP Specification](https://github.com/swarmchat/scp-spec)
- [scp-core](https://github.com/swarmchat/scp-core)
- [scp-relay](https://github.com/swarmchat/scp-relay)
- [scp-cli](https://github.com/swarmchat/scp-cli)
- [scp-sdk-python](https://github.com/swarmchat/scp-sdk-python)
