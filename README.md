# Lightning-sentinel
### Description:
- A telegram bot written in rust that will monitor a lighting node over tor. It does this through calling the node's REST API and sending notifications to a telegram private channel with the node administrator

### Privacy information and reason for this bot:
- The need to build this came after trying out the lightning-watch telegram bot and being frustrated that in it's ideal state it requires the user to lock up funds in a channel with it. Maybe this was a lot of effort to get around that problem, but we shall see.
- As the source code shows, the only data stored on a connected node is it's REST API url over tor and the provided macaroon. This means the bot has no idea about the IP of the nodes it's talking with. If anyone looking over this code has suggestions on increasing privacy of users, please reach out.

***NOTE: CURRENTLY ONLY SUPPORTS LND, BUT THERE ARE PLANS TO SUPPORT OTHER IMPLEMENTATIONS AS WELL IF PEOPLE FIND THIS USEFULL***

## To Use Existing Telegram Monitoring Bot:
### Requirements:
- Have an LND lightning node setup
- Have a REST API running on the same machine as the node that understands the "/getInfo" endpoint & is hosted on a static tor address
    - If you are running raspiblitz, one comes out of the box & this is what was used during development of this tool 
    - If you have another lnd node implementation, take a look at these options (NOTE: YOU NEED A STATIC ONION ADDRESS FOR THE REST API TO USE THIS TOOL!):
        - https://www.lightningnode.info/technicals/lightning.connect

### How to use:
- Create a private chat with the telegram bot @t.me/llightning_sentinel and send `\help`
- Create macaroon on your lightning node command line with access to "/getInfo":
   ` lncli bakemacaroon uri:/lnrpc.Lightning/GetInfo `
- Respond to the Bot's `\start` command with the tuple `(<lightning_node_tor_address>,<macaroon>)`, 
    ex:
        `(https://wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXXJJJJJJJJJJJJ.onion:8080, XXXXXXXXXXX...)`
---------------------------------------------------------------------------------------------------------------------------

# STOP HERE UNLESS YOU WANT TO ADMIN YOUR OWN BOT!!



## To Host Your Own Telegram Monitoring Bot:

### Requirements
- Have a remote server setup with ubuntu 20.04, compile/build from source using cargo or install from the release binary
- Create a new bot with telegram, following these instructions to create a new bot with botfather
    - https://medium.com/shibinco/create-a-telegram-bot-using-botfather-and-get-the-api-token-900ba00e0f39

### How to run:
- Build the binary from source using the Dockerfile or download the static binary from the github release: 
    - `docker build . & docker-compose run lightning-sentinel`
- Download binary: `wget https://github.com/tee8z/lightning-sentinel/releases/download/initial-release/lightning-sentinel`
- Copy the built binary to where you would like to run the service
- Don't forget to make the binary excutable with: `sudo chmod +x lightning-sentinel`
- Create a Settings.toml file from the Settings.default.toml in the directory it will be running in
- Add your telegram bot ID recieved from botfather 
- Then, go to `/etc/systemd/system` and create the following file, name `lightning-sentinel.service`:
- NOTE: If you'd like to run this under a different user than root, make sure to update the paths for WorkingDirectory & ExecStart


```
[Unit]
  Description=Telegram Bot monitoring lightning nodes 
  After=network.target 
  StartLimitIntervalSec=0 

[Service] 
 Type=simple 
 Restart=always 
 RestartSec=1 
 User=root 
 WorkingDirectory=~ 
 ExecStart=/root/lightning-sentinel 

[Install]  
  WantedBy=multi-user.target
```

- save the file, then run:
```
systemctl enable lightning-sentinel
systemctl daemon-reload
systemctl start lightning-sentinel

```
- Check the status with `systemctl status lightning-sentinel`, should be a green dot next to it now
- Proceed to register with the bot the same way as if you were using the existing bot

 ***NOTE: DO NOT HAVE THIS RUNNING ON THE SAME POWER SOURCE AS WHERE YOUR NODE IS RUNNING (WOULD DEFEATE THE PURPOSE OF THE BOT) ***
- How to set up lnconnect over tor, and host your rest API over it:
- https://github.com/openoms/bitcoin-tutorials/blob/master/Zap_to_RaspiBlitz_through_Tor.md


#### If you found this tool useful and feel like buying me a coffee/beer please donate with the button below (tor link):
[![](img/lightningPay.png)](https://qyalyxun6rwd6rguzjic2wycsumt4rv4q4sswyc2cbehnq6wokvivgad.onion/api/v1/invoices?storeId=9MoCExvosJ7hGKE4WQpb6xqAnZkSRrH5CQXUkPjqq9h&price=5&currency=USD)

TODO: 
- make sure can work with c-lightning and raw LND & a rest api running
- add pubkey/signing schema to verify the source code here is what the bot you are talk with is 