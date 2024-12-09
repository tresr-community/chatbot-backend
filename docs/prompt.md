# Prompt

The _Ron Jay_ AI prompt.

## Pre-requisites

- Download the whitepaper

```bash
wget --mirror --convert-links --adjust-extension --page-requisites --no-parent http://docs.nftreasure.com
```

- Convert the whitepaper to markdown

```bash
mkdir markdown

for FILE in $(find docs.nftreasure.com -name "*.html");
do

  echo "Converting $FILE to markdown"
  pandoc -s $FILE -o markdown/${FILE%.html}.md

done
```

## AI Prompt

TODO: Prompt goes here....
