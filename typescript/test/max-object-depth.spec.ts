import { ASLANParser } from '../src/aslan-parser';

describe('ASLANParser maxObjectDepth', () => {
  describe('Current behavior - demonstrating the issue', () => {
    // Case 1: Works fine with collapse enabled - content on same line
    test('collapse enabled: nested objects with inline content work correctly', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
      });
      const result = parser.parse(
        `[asland_edit1][aslano]
[asland_text]New text
[aslano]

[asland_edit2][aslano]
[asland_text]New text
[aslano]`,
      );
      expect(result).toEqual({
        _default: null,
        edit1: {
          text: 'New text\n',
        },
        edit2: {
          text: 'New text\n',
        },
      });
    });

    // Case 2: The problematic case - empty text field causes edit2 to get truncated
    test('collapse enabled: empty field followed by second edit gets truncated (BUG)', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
      });
      const result = parser.parse(
        `[asland_edit1][aslano]
[asland_text]
[aslano]

[asland_edit2][aslano]
[asland_text]This will get ignored
[aslano]`,
      );
      // This demonstrates the BUG: edit2 gets nested inside edit1.text
      // because whitespace collapse makes getObjectSafeLatestResult() return falsy
      expect(result).toEqual({
        _default: null,
        edit1: {
          text: {
            edit2: {
              text: 'This will get ignored\n',
            },
          },
        },
      });
    });

    // Case 3: Potential issue with collapse disabled if model puts o on new line
    test('collapse disabled: newline before aslano causes object creation to fail (BUG)', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: false,
      });
      const result = parser.parse(
        `[asland_edit1]
[aslano]
[asland_text]Some text
[aslano]`,
      );
      // The newline becomes content for edit1, so [aslano] closes instead of opening
      expect(result).toEqual({
        _default: null,
        edit1: '\n',
        text: 'Some text\n',
      });
    });

    // This case works with collapse disabled because the newline provides content
    test('collapse disabled: empty field followed by second edit works', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: false,
      });
      const result = parser.parse(
        `[asland_edit1][aslano]
[asland_text]
[aslano]

[asland_edit2][aslano]
[asland_text]This should not get ignored
[aslano]`,
      );
      expect(result).toEqual({
        _default: null,
        edit1: {
          text: '\n',
        },
        edit2: {
          text: 'This should not get ignored\n',
        },
      });
    });
  });

  describe('maxObjectDepth feature - expected behavior', () => {
    // With maxObjectDepth: 1, the o operator should be binary:
    // - At object depth 0: [aslano] always creates a new object (push to stack)
    // - At object depth 1: [aslano] always closes the current object (pop from stack)
    // This eliminates ambiguity from whitespace content

    test('maxObjectDepth=1: empty field should not cause truncation', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
        maxObjectDepth: 1,
      });
      const result = parser.parse(
        `[asland_edit1][aslano]
[asland_text]
[aslano]

[asland_edit2][aslano]
[asland_text]This should NOT get ignored
[aslano]`,
      );
      // With maxObjectDepth: 1:
      // - [aslano] after edit1: depth 0 -> creates object (depth becomes 1)
      // - [aslano] after text: depth 1 -> closes object (depth becomes 0)
      // - [aslano] after edit2: depth 0 -> creates object (depth becomes 1)
      // - [aslano] after text: depth 1 -> closes object (depth becomes 0)
      expect(result).toEqual({
        _default: null,
        edit1: {
          text: '\n',
        },
        edit2: {
          text: 'This should NOT get ignored\n',
        },
      });
    });

    test('maxObjectDepth=1: newline before aslano follows existing logic at depth 0', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: false,
        maxObjectDepth: 1,
      });
      const result = parser.parse(
        `[asland_edit1]
[aslano]
[asland_text]Some text
[aslano]`,
      );
      // With maxObjectDepth: 1 and collapseObjectStartWhitespace: false:
      // At depth 0 < maxDepth, existing create/close logic still applies
      // The newline becomes content for edit1, so existing logic sees non-empty
      // and tries to close (which is noop at root). Object is NOT created.
      // NOTE: This is expected - maxObjectDepth only limits depth, doesn't change
      // create/close logic at depth < maxDepth. To fix this case, use
      // collapseObjectStartWhitespace: true instead.
      expect(result).toEqual({
        _default: null,
        edit1: '\n',
        text: 'Some text\n',
      });
    });

    test('maxObjectDepth=1: inline version should produce same structure as multiline', () => {
      const parserInline = new ASLANParser({
        collapseObjectStartWhitespace: true,
        maxObjectDepth: 1,
      });
      const resultInline = parserInline.parse(
        `[asland_edit1][aslano][asland_text]content[aslano][asland_edit2][aslano][asland_text]more content[aslano]`,
      );

      const parserMultiline = new ASLANParser({
        collapseObjectStartWhitespace: true,
        maxObjectDepth: 1,
      });
      const resultMultiline = parserMultiline.parse(
        `[asland_edit1][aslano]
[asland_text]content
[aslano]

[asland_edit2][aslano]
[asland_text]more content
[aslano]`,
      );

      // Both should have the same structure (ignoring whitespace in content)
      expect(resultInline).toEqual({
        _default: null,
        edit1: {
          text: 'content',
        },
        edit2: {
          text: 'more content',
        },
      });

      expect(resultMultiline).toEqual({
        _default: null,
        edit1: {
          text: 'content\n',
        },
        edit2: {
          text: 'more content\n',
        },
      });
    });

    test('maxObjectDepth=1: second aslano should close instead of nesting deeper', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
        maxObjectDepth: 1,
      });

      // Attempting to nest more than 1 level deep
      const result = parser.parse(
        `[asland_a][aslano][asland_b][aslano][asland_c]value[aslano][aslano]`,
      );

      // With maxObjectDepth: 1:
      // - [aslano] after a: depth 0, 'a' is empty -> creates object for a (depth becomes 1)
      // - [asland_b]: sets current key to 'b' inside 'a', but no content written yet
      // - [aslano] after b: depth 1 >= maxDepth -> always close (depth becomes 0)
      //   Note: 'b' never had content written, so 'a' ends up empty {}
      // - [asland_c]: sets key to 'c' at top level
      // - value: c = 'value'
      // - [aslano] after c/value: depth 0, follows normal logic (close noop at root)
      // - final [aslano]: same, noop
      expect(result).toEqual({
        _default: null,
        a: {},
        c: 'value',
      });
    });

    test('maxObjectDepth=1: works with arrays inside objects', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
        maxObjectDepth: 1,
      });

      const result = parser.parse(
        `[asland_edit][aslano][asland_items][aslana][asland]item1[asland]item2[aslano]`,
      );

      // Array nesting should still work, only object depth is limited
      expect(result).toEqual({
        _default: null,
        edit: {
          items: ['item1', 'item2'],
        },
      });
    });

    test('maxObjectDepth=0: no objects allowed, aslano always ignored at root', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
        maxObjectDepth: 0,
      });

      const result = parser.parse(
        `[asland_edit][aslano][asland_text]content[aslano]`,
      );

      // With maxObjectDepth: 0, [aslano] always tries to close (depth 0 >= 0)
      // At depth 0, closing is a no-op (nothing to pop from stack)
      // So [aslano] is effectively ignored, and 'edit' never gets content written
      // The next data delimiter 'text' gets the content
      expect(result).toEqual({
        _default: null,
        text: 'content',
      });
    });

    test('maxObjectDepth=2: allows two levels of nesting', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
        maxObjectDepth: 2,
      });

      const result = parser.parse(
        `[asland_a][aslano][asland_b][aslano][asland_c]value[aslano][aslano]`,
      );

      // With maxObjectDepth: 2:
      // - [aslano] after a: depth 0 < 2, 'a' is empty -> creates object (depth 1)
      // - [aslano] after b: depth 1 < 2, 'b' is empty -> creates nested object (depth 2)
      // - [aslano] after c/value: depth 2 >= 2 -> closes (depth 1)
      // - final [aslano]: depth 1 < 2, but at this point we've returned to parent with non-empty content
      //   Following normal logic, this should close (depth 0)
      expect(result).toEqual({
        _default: null,
        a: {
          b: {
            c: 'value',
          },
        },
      });
    });

    test('maxObjectDepth undefined: default behavior (unlimited depth)', () => {
      const parser = new ASLANParser({
        collapseObjectStartWhitespace: true,
        // maxObjectDepth not set - should behave as before
      });

      const result = parser.parse(
        `[asland_a][aslano][asland_b][aslano][asland_c][aslano][asland_d]value`,
      );

      // Without maxObjectDepth, deep nesting should work as before
      expect(result).toEqual({
        _default: null,
        a: {
          b: {
            c: {
              d: 'value',
            },
          },
        },
      });
    });
  });
});
