# How to deploy

This is an example of deploying the application onto a Raspberry Pi Linux installation

## General assumptions

You can connect with ssh to the Linux os.
You have a work directory as a user.

```sh
cd ~
mkdir work
```

## Clone the repository

```sh
cd ~/work
git clone https://github.com/dezGusty/rs-weight-tracker.git
```

## Build the application

Building the application

```sh
cd ~/work/rs-weight-tracker
cargo build --release
```

## Check that you can run the application

```sh
cd ~/work/rs-weight-tracker/
./target/release/rs-weight-tracker
```
Quit the application (CTRL + C)

### Get some data

Note: you may need to run the migration, 

```sh
diesel setup
diesel migration run
cargo run --release --bin import_weights ./data/sample_data.json
cargo run --release --bin show_weight_interval 2022-03-01 2023-03-01
```

## Configure the daemon

Configuring a daemon makes life easier.
The information is based on the examples from <https://www.shellhacks.com/systemd-service-file-example/>

Choose a name for the service. E.g. `rs-weight-tracker`
Create the file and edit it with a text editor, such as nano:

```sh
cd /etc/systemd/system
sudo touch rs-weight-tracker.service
sudo nano rs-weight-tracker.service
```

Adjust the content. For me, the user is `gus`. You could change it

```ini
[Unit]
Description=A simple service for a weight tracker
After=network.target

[Service]
Type=simple
User=johnybravo
ExecStart=/home/johnybravo/work/rs-weight-tracker/target/release/rs-weight-tracker
Restart=always
RestartSec=2
TimeoutStartSec=0
WorkingDirectory=/home/johnybravo/work/rs-weight-tracker

[Install]
WantedBy=multi-user.target
```

Reload the daemons, enable the new daemon and start it

```sh
sudo systemctl daemon-reload
sudo systemctl enable ui-weight-tracker
sudo systemctl start ui-weight-tracker
```

## Configure the proxy

Ok, so you have a running application, but how do you reach it ?

### VNC

1. You could connect to the system using a VNC connection.

```sh
vncserver
```

2. Open a connection with a VNC Client
3. On the Host system, open a browser and open the running application port.
(E.g. <http://localhost:23128/>)

### Proxy it

This assumes you have nginx installed.

```sh
cd /etc/nginx
sudo nano nginx.conf
```
OR
```sh
sudo nano /etc/nginx/nginx.conf
```

Edit the nginx configuration file. Please note that the configuration has sections and sub-sections such as

```txt
events {

}
http {
  server {

  }
}
```

you will have to change settings in the http / server section

```nginx
                location /weight-tracker/ {
                        proxy_pass http://127.0.0.1:14280;
                        # Strip the /weight-tracker prefix from the URL
                        rewrite /weight-tracker(.*) $1 break;
                }
```

Restart the nginx service, and make sure that no errors are reported.

```sh
sudo systemctl restart nginx
systemctl status nginx.service
```
