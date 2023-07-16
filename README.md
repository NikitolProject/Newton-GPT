# Newton-Chat-GPT (Discord bot)
![alt text](https://github.com/NikitolProject/Newton-GPT/raw/master/assets/demo.png)
---
## Introduction
### This bot implements the basic functionality of a regular ChatGPT (in the form of creating new chats, communicating within them with support for remembering the context, as well as with the ability to change the current model). The management interface is in the terminal, and currently has very limited functionality. The project itself is still under development, and this is only its MVP version.
---
# Installation
---
## Production:
### To install a ready-made bot, all you need to do is download its binary file, and add and customize the .env file, according to the .env.template.
### After that, just type:
```bash
./discord_gpt_bot
```
### And the program itself will create all the other files it needs.
---
## Dev:
### Similar to the Production version, in Dev you need to create and populate an .env file in the root folder of the project to get started.
### After that, all you have to do is enter these commands:
```bash
make build
./discord_gpt_bot
```
### And the program build will already be ready.
