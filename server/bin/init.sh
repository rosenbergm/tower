echo "Welcome to the Tower installer!"
echo "We will now install the Tower application and a Caddy server instance."
echo ""

# Check if the user is root
if [ "$EUID" -ne 0 ]
  then echo "You need to run this script as root."
  exit
fi

# Check if the user has docker installed
if ! [ -x "$(command -v docker)" ]; then
  echo "Docker is not installed. Please install Docker and try again."
  exit
fi

mkdir -p /tmp/tower

cat > /tmp/tower/Caddyfile.default <<- EOM
{
    admin 0.0.0.0:2019
}
EOM

sudo mkdir -p /etc/tower/caddy/data

sudo docker network create -d bridge tower_network

sudo docker run --name tower_caddy --expose 2019 -p 80:80 -p 443:443 --network tower_network -v /tmp/tower/Caddyfile.default:/etc/caddy/Caddyfile -v /etc/tower/caddy/data:/data caddy
