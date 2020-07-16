# Vivec

This is a library meant for parsing and performing actions in Skyrim plugin files. Skyrim is the _main_ target, but supporting other games using the same engine is also a goal.
The UESP wiki has been a great help in writing this.
The library is in a very Alpha state right now, and _everything_ is subject to change.

### Supported Records

The game sorts things at the top level into Records or Groups. Groups also contain records, just grouping them together for easier access by an editor (so far it's just made my code more complicated..).
There is quite a number of records, see the (Record Types)[https://en.uesp.net/wiki/Tes5Mod:Mod_File_Format#Record_Types].

- AACT (+Group): Action
- ACHR: Actor Reference
- ACTI (+Group):
- ADDN (+Group): Addon Node
- ALCH (+Group):
- TES4: Plugin Info

### Contributing

Contributions are welcome. Currently most of what is needed is more Records being supported, and various todos in the source code being fixed (though, some are more 'think about what to do with this once I have a more complete setup').
If you want to implement a record, then choose one from later in the list as I am attempting to go through the list in alphabetical order.
Records and fields that implement `FromField`/`FromRecord`, `Parse`, `TypeNamed`/`StaticTypeNamed`, `DataSize`/`StaticDataSize`, and `Writable` should do so in that order. Implementations of functions on that record/field/thing that are not trait-related should go before all of those.
