# Sprocket Web Components

Collection of web components for use in the Sprocket documentation.

## Build Instructions

> Note: requires Node.js 22.10.0 or higher

```bash
npm install
npm run build
```

The built files will be placed in the `../theme/assets` directory.

Optionally, you may also use `npm run dev` to watch for changes and automatically rebuild the files.

## Usage

#### Sprocket Code

Adds syntax highlighting to code blocks, supports WDL and Rust languages.

```html
<sprocket-code language="rust">
  fn main() {
    println!("Hello, world!");
  }
</sprocket-code>

<sprocket-code language="wdl">
version 1.0

workflow count_lines {
  input { File input_file }
  call Count { input: file = input_file }
  output { Int num_lines = Count.num_lines }
}

task Count {
  input { File file }
  command { wc -l ${file} | awk '{print $1}' }
  output { Int num_lines = read_int(stdout()) }
}
</sprocket-code>
```

#### Sprocket Tooltip

Shows a tooltip with the specified content.

```html
<sprocket-tooltip position="bottom" content="This will be shown under it">
  <span>Hover over me!</span>
</sprocket-tooltip>
```

Available positions are: `top`, `bottom`, `left`, `right`.
