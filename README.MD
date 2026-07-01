# Setup
The moody executable should be placed in the "moody" folder. The version of Alacritty you are using should be named "alacritty.exe" and placed in the "alacritty" folder. An alacritty.toml file should be in that folder as well, containing the [configuration](https://alacritty.org/config-alacritty.html).
```
Vigilance Tester Directory
+-- launcher.exe
|-- moody
|   +-- moody.exe
|
|-- alacritty
|   +-- alacritty.exe
|   +-- alacritty.toml
```
# Running
Use the launcher executable to run the test. The launcher is used to open the terminal emulator Alacritty, which is an emulator that allows a greater degree of customization (at the command line) than default windows terminals. The alacritty.toml file found in the "alacritty" folder can be edited and will be automatically passed in to the calling terminal, which can allow for things like changing the dimensions and opening the program in fullscreen.
The settings of each test run will be saved to a "config.txt" file in the moody folder, which will save the last setting used and populate the program with the contents of the file on restart.

# Output
An output folder will be created at the same level as the launcher. All results will be placed there, in the format of year-month-day_hits-misses-falsehits.csv. It will contain a list of three kinds of events, as well as the times those events occurred relative to the start of the test beginning (not the start of the program opening).

