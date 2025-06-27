# IPChanger2

IPChanger2 is a simple, automated tool designed to change your set of IP addresses. It allows you to easily rotate your IP address to access subnets or for network testing purposes.

## Features

- Simple command-line interface.
- Lightweight and easy to configure.

## Initial Setup

When you first launch the application, it will display the MAC addresses of all network interfaces on your system and automatically create a `config.toml` file. Simply open this configuration file and enter the MAC address of the network interface you want to manage.

## Running the Application

After configuring the app, run it with administrator privileges. The IPChanger command prompt will appear, allowing you to use the following commands:

**Available Commands:**
- **`l`** - Display current IP addresses
- **`a`** - Add an IP address with subnet mask (example: `a192.168.2.22/24`)
- **`d`** - Remove an IP address by index number (example: `d2` removes the IP address at position 2)
