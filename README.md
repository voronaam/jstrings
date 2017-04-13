# jstrings
Java Strings Extractor

The purpose of this simple tool is to find all string litetals in a compiled Java code.
It searches for the most common locations and prints each foudn string to stdout.
The tool is not attempting to re-escape the strings and multi-line strings may be
outputed as-is - as multilines.

Currently implemented search locations:
- String constants in class files
- Values in java properties files

Future search locations:
- XML files.
- YAML files

I wrote this tool to automatically inspect a JAR file we are about to distribute
to check if any information we do not want to distribute ended up in the artifact.
It is meant to catch accidental inclusion of unit and integration test data into
production artifacts.

## Why Rust?

It was possible to read jar files from java, load classes and inspect them using 
reflection. However, that would invoke static initializers on those classes and
those may fail. It may also consume more memory and CPU while doing this.

Using Java Class parser from another language allows to analyze the class
file "at rest". Which may also be valuable in some cases.
