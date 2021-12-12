
To get address and access to node
1) make sure port forwarding 8080 to raspi on router
2) raspiblitz:
    (gives you the address of the REST API over tor)
    > MENU > CONNECT > MOBILE > FULLYNODED_LND > Continue > Console QRcode 
    - Copy and paste:
    using TOR --> host wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXXJJJJJJJJJJJJ.onion port 8080

    lndconnect://wkdirllfgoofflfXXXXXXXXXXXXXXXXXXXXXXXXXXXX.onion:8080?macaroon=XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY

    NOTE: TOR ADDRESS USED HERE ARE FAKE, Nothing will happen if you try to use them


 - Create a new bot with telegram
https://medium.com/shibinco/create-a-telegram-bot-using-botfather-and-get-the-api-token-900ba00e0f39

- Or use the one already created @t.me/llightning_sentinel

TODO:
- need encrytped file to store key/value of user to tor address for sending notification messages (if the file ever gets taken then there is a mapping between user # and tor address, not terrible but want to make it so that's not possible, some sort of key schema?)
- would like to show if node is up or down & what channels are active when checking
- implement long pulling to telegram bot (will get booted if go to prod without it)
- create structure for channels to be many-to-many telegram - lnd messages per user