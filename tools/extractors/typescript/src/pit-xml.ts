export interface XmlElement {
  name: string;
  attributes: Record<string, string>;
  children: XmlElement[];
  text: string;
}

export function parseXml(source: string): XmlElement {
  const input = source.replace(/^\uFEFF/, '');
  const roots: XmlElement[] = [];
  const stack: XmlElement[] = [];
  let offset = 0;

  while (offset < input.length) {
    if (input[offset] !== '<') {
      const end = input.indexOf('<', offset);
      const next = end === -1 ? input.length : end;
      appendText(stack, roots, decodeEntities(input.slice(offset, next)));
      offset = next;
      continue;
    }
    if (input.startsWith('<!--', offset)) {
      const end = input.indexOf('-->', offset + 4);
      if (end === -1) malformed('unterminated comment');
      offset = end + 3;
      continue;
    }
    if (input.startsWith('<![CDATA[', offset)) {
      const end = input.indexOf(']]>', offset + 9);
      if (end === -1) malformed('unterminated CDATA section');
      appendText(stack, roots, input.slice(offset + 9, end));
      offset = end + 3;
      continue;
    }
    if (input.startsWith('<?', offset)) {
      const end = input.indexOf('?>', offset + 2);
      if (end === -1) malformed('unterminated processing instruction');
      const target = input.slice(offset + 2, end).trim().split(/\s/, 1)[0];
      if (target !== 'xml' || roots.length > 0 || stack.length > 0) {
        malformed(`unsupported processing instruction ${target || '(empty)'}`);
      }
      offset = end + 2;
      continue;
    }
    if (input.startsWith('<!', offset)) {
      malformed('document declarations are not supported');
    }
    if (input.startsWith('</', offset)) {
      const end = input.indexOf('>', offset + 2);
      if (end === -1) malformed('unterminated closing tag');
      const name = input.slice(offset + 2, end).trim();
      if (!xmlName(name)) malformed(`invalid closing tag ${name || '(empty)'}`);
      const open = stack.pop();
      if (!open || open.name !== name) malformed(`unexpected closing tag ${name}`);
      offset = end + 1;
      continue;
    }

    const end = tagEnd(input, offset + 1);
    const body = input.slice(offset + 1, end);
    const selfClosing = /\/\s*$/.test(body);
    const content = selfClosing ? body.replace(/\/\s*$/, '') : body;
    const element = openingTag(content);
    if (stack.length === 0) roots.push(element);
    else stack[stack.length - 1].children.push(element);
    if (!selfClosing) stack.push(element);
    offset = end + 1;
  }

  if (stack.length > 0) malformed(`unclosed tag ${stack[stack.length - 1].name}`);
  if (roots.length !== 1) malformed('document must contain exactly one root element');
  return roots[0];
}

function openingTag(source: string): XmlElement {
  const nameMatch = /^\s*([A-Za-z_][A-Za-z0-9_.:-]*)/.exec(source);
  if (!nameMatch) malformed('opening tag has no valid name');
  const name = nameMatch[1];
  const attributes: Record<string, string> = {};
  let offset = nameMatch[0].length;
  while (offset < source.length) {
    const whitespace = /^\s+/.exec(source.slice(offset));
    if (!whitespace) malformed(`invalid attributes on ${name}`);
    offset += whitespace[0].length;
    if (offset === source.length) break;
    const attribute = /^([A-Za-z_][A-Za-z0-9_.:-]*)\s*=\s*(["'])(.*?)\2/s
      .exec(source.slice(offset));
    if (!attribute) malformed(`invalid attributes on ${name}`);
    if (Object.hasOwn(attributes, attribute[1])) {
      malformed(`duplicate attribute ${attribute[1]} on ${name}`);
    }
    attributes[attribute[1]] = decodeEntities(attribute[3]);
    offset += attribute[0].length;
  }
  return { name, attributes, children: [], text: '' };
}

function tagEnd(source: string, offset: number): number {
  let quote = '';
  for (let index = offset; index < source.length; index++) {
    const value = source[index];
    if (quote) {
      if (value === quote) quote = '';
    } else if (value === '"' || value === "'") {
      quote = value;
    } else if (value === '>') {
      return index;
    }
  }
  malformed('unterminated opening tag');
}

function appendText(stack: XmlElement[], roots: XmlElement[], value: string): void {
  if (stack.length === 0) {
    if (value.trim().length > 0 || roots.length > 1) malformed('text outside root element');
    return;
  }
  stack[stack.length - 1].text += value;
}

function decodeEntities(value: string): string {
  const entity = /&(?:#x[0-9A-Fa-f]+|#[0-9]+|amp|lt|gt|quot|apos);/y;
  for (let index = value.indexOf('&'); index !== -1; index = value.indexOf('&', index + 1)) {
    entity.lastIndex = index;
    if (!entity.test(value)) malformed(`invalid entity at character ${index}`);
  }
  return value.replace(/&(#x[0-9A-Fa-f]+|#[0-9]+|amp|lt|gt|quot|apos);/g, (_, entity: string) => {
    if (entity === 'amp') return '&';
    if (entity === 'lt') return '<';
    if (entity === 'gt') return '>';
    if (entity === 'quot') return '"';
    if (entity === 'apos') return "'";
    const radix = entity.startsWith('#x') ? 16 : 10;
    const digits = entity.slice(radix === 16 ? 2 : 1);
    const codepoint = Number.parseInt(digits, radix);
    if (!Number.isInteger(codepoint) || codepoint > 0x10ffff) {
      malformed(`invalid entity &${entity};`);
    }
    return String.fromCodePoint(codepoint);
  });
}

function xmlName(value: string): boolean {
  return /^[A-Za-z_][A-Za-z0-9_.:-]*$/.test(value);
}

function malformed(message: string): never {
  throw new Error(`malformed PIT XML: ${message}`);
}
