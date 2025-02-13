#!/bin/sh

if [ "$NO_COLOR" = "" ]; then
   GREEN="\033[32m";
   CYAN="\033[36m";
   YELLOW="\033[33m";
   BRIGHT_RED="\033[91m";
   RESET="\033[0m";
   BLUE="\033[34m";
else
   GREEN="";
   CYAN="";
   YELLOW="";
   BRIGHT_RED="";
   RESET="";
   BLUE="";
fi
