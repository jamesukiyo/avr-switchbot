# AVR Switchbot

Experimenting with embedded Rust. A home-made "switchbot" that I use to turn on
a computer tucked away in my room.

It isn't very sophisticated and just accepts any input from a generic infrared
remote. Upon pressing any button on the IR remote, the Arduino rotates a servo
which in turn presses the power button on the computer, the servo then rotates
back to the starting position.

More details about hardware, pins and such can be found in `src/main.rs`.

A simple circuit diagram can be seen in `images/diagram.png`.

`flake.nix` should contain all necessary dependencies to get started.

This would not be possible without the excellent work being done at
[Rahix/avr-hal](https://github.com/rahix/avr-hal).

## License

```
The MIT License (MIT)

Copyright (c) 2025-present PlumJam <git@plumj.am>

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```
