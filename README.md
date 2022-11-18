# Lightning-sentinel
### Description:
- A nostr bot to monitor a lighting node. It does this through calling the node's REST API and sending an encrypted notification through a defined nostr-relays.

### Privacy information and reason for this bot:
- The need to build this came after trying out the lightning-watch telegram bot and being frustrated that in it's ideal state it requires the user to lock up funds in a channel with it. Maybe this was a lot of effort to get around that problem, but we shall see.
- As the source code shows, the only data stored on a connected node is it's REST API url and the provided macaroon. When the REST API is hosted at a tor url this bot has no idea about the IP of the nodes it's talking with, but it does allow for clearnet if someone would rather use that instead. If anyone looking over this code has suggestions on increasing privacy of users, please reach out.

***NOTE: CURRENTLY ONLY SUPPORTS LND, BUT THERE ARE PLANS TO SUPPORT OTHER IMPLEMENTATIONS AS WELL IF PEOPLE FIND THIS USEFULL***

## To Use Existing Telegram Monitoring Bot:
### Requirements:
- Have an lightning node setup
- Have a REST API running on the same machine as the node that understands the "/v1/getinfo" endpoint & is hosted on a static address
    - If you are running raspiblitz, one comes out of the box & this is what was used during development of this tool 
    - If you have another lnd node implementation, take a look at these options (NOTE: YOU NEED A STATIC ONION ADDRESS FOR THE REST API TO USE THIS TOOL!):
        - https://www.lightningnode.info/technicals/lightning.connect

### How to use:
- Create a  `\help`
- Create macaroon on your lightning node command line with access to "/v1/getinfo":
   ` lncli bakemacaroon uri:/lnrpc.Lightning/GetInfo `
- Respond to the Bot's `\start` command with the tuple `<lightning_node_address>, <macaroon>`, 
    ex:
        `https://wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXXJJJJJJJJJJJJ.onion:443, XXXXXXXXXXX...`
        `https://wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXXJJJJJJJJJJJJ.onion:8080, XXXXXXXXXXX...`
        `https://192.34.21.1:4801, XXXXXXXXXXX...`
---------------------------------------------------------------------------------------------------------------------------

# STOP HERE UNLESS YOU WANT TO ADMIN YOUR OWN BOT!!


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

