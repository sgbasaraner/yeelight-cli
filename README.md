
# yeelight-cli

cargo install yee

This is a cross-platform command line client for Xiaomi's Yeelight. It is a very low-level implementation, and I believe it is possible to build something on top of this, maybe even a GUI. This still needs a lot of work to be full-featured, however, I believe it is usable at this stage.

# Usage

You can use any method in [Yeelight Specification](http://www.yeelight.com/download/Yeelight_Inter-Operation_Spec.pdf) as long as your device supports it, as this program is essentially a wrapper around it.

You need to enable LAN Control using the official Yeelight mobile app before attempting to use this, or any other 3rd party client.

# Using with command line arguments

You can also use this program with command line arguments, this is useful for performing simple operations such as turning the light off and on. To use with command line arguments, you need to know your light bulb's assigned name, or if you haven't assigned a name yet, you have to assign it.

For example:

    yee mybulb toggle

would look for a lightbulb named "mybulb", and toggle it.

# Intent

I encountered many difficulties developing this as the only source codes I was able to found was from non-English speaking developers and it made reading comments and grasping difficult parts hard. I believe this can act as a guide for developers who are new to writing programs for this kind of devices, especially Yeelight.
