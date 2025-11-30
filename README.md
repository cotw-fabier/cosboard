Cosboard is a libcosmic based software keyboard with a focus on script defined layouts, scripting, STT, and swipe gestures.

Cosboard is meant to act as a natural extension of the app experience in Cosmic to provide keyboard interactions when on touch enabled devices. Libcosmic does this in a number of ways:

1.) Layouts are defined via JSON files. Layouts are broken down by Panels -> Rows -> Keys. 

- Panels can move around or disappear depending on the keyboard size (for example, a numpad may only be visible in landscape view because portrait doesn't have enough space to display it).
- Rows allow the user to organize keys. Leys are given a width layout to allow for some keys to be larger than others. The entire row is given a height layout to allow for some rows to be taller than others. Rows heights can also change based on available space both width and height wise to make keyboards responsive.
- Keys allow the definition of several actions on each key.
  - Key press
  - Long press
  - Long Press + Swipe <direction>
- Key Actions can be a specific key character (based on UTF-8 standard). But actions can also be overridden by scripts. Cosboard has a small scripting engine built in to allow the combination of keys or a combination of other functions such as moving between windows, activating other system features, or executing user-made scripts in the userspace terminal.
- Keys will default to their standard action when performing swipe actions. If the action does not include a standard key then the key is omitted from swipe gesture lookups.

Cosboard defaults to acting like a floating applet which allows it to operate in both tiled and float environments. It should default to displaying always above other apps and should be easily resizable by clicking to drag. Cosboard also accepts both mouse and touch gestures which both react the same way in the app. Allowing people with mice and/or touchscreens to utilize the keyboard.

The keyboard should be able to read in JSON files to define new layouts. There should also be some smart widgets which the keyboard supports such as word suggestions for predicted autocomplete and next word prediction.

As far as autocomplete, next word completion, STT, and other smart features. there should be some smart widgets which can be placed into the keyboard layout files. There should be some default prediction dictionaries but there should also be a user-prediction dictionary which should be populated as the user adds manual words.
