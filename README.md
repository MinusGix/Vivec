# Vivec

This is a library meant for parsing and performing actions in Skyrim plugin files. Skyrim is the _main_ target, but supporting other games using the same engine is also a goal.
The UESP wiki has been a great help in writing this.

### Supported Records

The game sorts things at the top level into Records or Groups. Groups also contain records, just grouping them together for easier access by an editor (so far it's just made my code more complicated..).
There is quite a number of records, see the (Record Types)[https://en.uesp.net/wiki/Tes5Mod:Mod_File_Format#Record_Types].

- AACT (+Group): Action
- ACHR: Actor Reference
- ACTI (+Group):
- ADDN (+Group): Addon Node
- ALCH (+Group):
- TES4: Plugin Info
