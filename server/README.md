## Development commands

```bash
docker run --rm --name tower_caddy -p 2019:2019 -p 5591:80 -p 2009:2009 --network tower_network -v $PWD/Caddyfile.default:/etc/caddy/Caddyfile -v $PWD/data:/data caddy
```