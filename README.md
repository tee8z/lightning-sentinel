
How to use:
1) Use the bot already setup with telegram, start a chat with @t.me/llightning_sentinel and send `\help`
2) Create a new bot with telegram:
    - following these instructions to create a new bot with botfather:
        - https://medium.com/shibinco/create-a-telegram-bot-using-botfather-and-get-the-api-token-900ba00e0f39
    - Add to Settings.toml:
        - TELEGRAM_BOT_ID
        - TELEGRAM_BASE_URL
    - Kick off this project on a linux machine somewhere (cloud/old machine with different power source) to listen/act on commands sent from telegram/lightning node


How to get data needed to register with the bot:
1) Make sure port forwarding 8080 to raspi on router is setup
2) Go to Raspiblitz menu after ssh to the machine, then:
    (gives you the address of the REST API over tor)
    > MENU > CONNECT > MOBILE > FULLYNODED_LND > Continue > Console QRcode 
    - Address will appear like this below the QR code:
        - host wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXXJJJJJJJJJJJJ.onion port 8080
3) Create macaroon:
   ` lncli bakemacaroon uri:/lnrpc.Lightning/GetInfo `
4) Response to the Bot's `\start` command with the tuple `(<lightning_node_tor_address>,<macaroon>)`, 
    ex:
        `(https://wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXXJJJJJJJJJJJJ.onion:8080, XXXXXXXXXXX...)`


TODO:
- would like to show if node is up or down & what channels are active when checking