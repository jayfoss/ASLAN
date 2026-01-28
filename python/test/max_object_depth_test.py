from aslan.aslan_parser import ASLANParser


class TestMaxObjectDepthCurrentBehavior:
    """Current behavior - demonstrating the issue"""

    def test_collapse_enabled_nested_objects_with_inline_content_work_correctly(self):
        """Case 1: Works fine with collapse enabled - content on same line"""
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
        })
        result = parser.parse(
            "[asland_edit1][aslano]\n"
            "[asland_text]New text\n"
            "[aslano]\n"
            "\n"
            "[asland_edit2][aslano]\n"
            "[asland_text]New text\n"
            "[aslano]"
        )
        assert result == {
            "_default": None,
            "edit1": {
                "text": "New text\n",
            },
            "edit2": {
                "text": "New text\n",
            },
        }

    def test_collapse_enabled_empty_field_followed_by_second_edit_gets_truncated(self):
        """Case 2: The problematic case - empty text field causes edit2 to get truncated (BUG)"""
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
        })
        result = parser.parse(
            "[asland_edit1][aslano]\n"
            "[asland_text]\n"
            "[aslano]\n"
            "\n"
            "[asland_edit2][aslano]\n"
            "[asland_text]This will get ignored\n"
            "[aslano]"
        )
        # This demonstrates the BUG: edit2 gets nested inside edit1.text
        # because whitespace collapse makes get_object_safe_latest_result() return falsy
        assert result == {
            "_default": None,
            "edit1": {
                "text": {
                    "edit2": {
                        "text": "This will get ignored\n",
                    },
                },
            },
        }

    def test_collapse_disabled_newline_before_aslano_causes_object_creation_to_fail(self):
        """Case 3: Potential issue with collapse disabled if model puts o on new line (BUG)"""
        parser = ASLANParser({
            'collapseObjectStartWhitespace': False,
        })
        result = parser.parse(
            "[asland_edit1]\n"
            "[aslano]\n"
            "[asland_text]Some text\n"
            "[aslano]"
        )
        # The newline becomes content for edit1, so [aslano] closes instead of opening
        assert result == {
            "_default": None,
            "edit1": "\n",
            "text": "Some text\n",
        }

    def test_collapse_disabled_empty_field_followed_by_second_edit_works(self):
        """This case works with collapse disabled because the newline provides content"""
        parser = ASLANParser({
            'collapseObjectStartWhitespace': False,
        })
        result = parser.parse(
            "[asland_edit1][aslano]\n"
            "[asland_text]\n"
            "[aslano]\n"
            "\n"
            "[asland_edit2][aslano]\n"
            "[asland_text]This should not get ignored\n"
            "[aslano]"
        )
        assert result == {
            "_default": None,
            "edit1": {
                "text": "\n",
            },
            "edit2": {
                "text": "This should not get ignored\n",
            },
        }


class TestMaxObjectDepthFeature:
    """maxObjectDepth feature - expected behavior
    
    With maxObjectDepth: 1, the o operator should be binary:
    - At object depth 0: [aslano] always creates a new object (push to stack)
    - At object depth 1: [aslano] always closes the current object (pop from stack)
    This eliminates ambiguity from whitespace content
    """

    def test_max_object_depth_1_empty_field_should_not_cause_truncation(self):
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
            'maxObjectDepth': 1,
        })
        result = parser.parse(
            "[asland_edit1][aslano]\n"
            "[asland_text]\n"
            "[aslano]\n"
            "\n"
            "[asland_edit2][aslano]\n"
            "[asland_text]This should NOT get ignored\n"
            "[aslano]"
        )
        # With maxObjectDepth: 1:
        # - [aslano] after edit1: depth 0 -> creates object (depth becomes 1)
        # - [aslano] after text: depth 1 -> closes object (depth becomes 0)
        # - [aslano] after edit2: depth 0 -> creates object (depth becomes 1)
        # - [aslano] after text: depth 1 -> closes object (depth becomes 0)
        assert result == {
            "_default": None,
            "edit1": {
                "text": "\n",
            },
            "edit2": {
                "text": "This should NOT get ignored\n",
            },
        }

    def test_max_object_depth_1_newline_before_aslano_follows_existing_logic_at_depth_0(self):
        parser = ASLANParser({
            'collapseObjectStartWhitespace': False,
            'maxObjectDepth': 1,
        })
        result = parser.parse(
            "[asland_edit1]\n"
            "[aslano]\n"
            "[asland_text]Some text\n"
            "[aslano]"
        )
        # With maxObjectDepth: 1 and collapseObjectStartWhitespace: False:
        # At depth 0 < maxDepth, existing create/close logic still applies
        # The newline becomes content for edit1, so existing logic sees non-empty
        # and tries to close (which is noop at root). Object is NOT created.
        # NOTE: This is expected - maxObjectDepth only limits depth, doesn't change
        # create/close logic at depth < maxDepth. To fix this case, use
        # collapseObjectStartWhitespace: True instead.
        assert result == {
            "_default": None,
            "edit1": "\n",
            "text": "Some text\n",
        }

    def test_max_object_depth_1_inline_version_same_structure_as_multiline(self):
        parser_inline = ASLANParser({
            'collapseObjectStartWhitespace': True,
            'maxObjectDepth': 1,
        })
        result_inline = parser_inline.parse(
            "[asland_edit1][aslano][asland_text]content[aslano][asland_edit2][aslano][asland_text]more content[aslano]"
        )

        parser_multiline = ASLANParser({
            'collapseObjectStartWhitespace': True,
            'maxObjectDepth': 1,
        })
        result_multiline = parser_multiline.parse(
            "[asland_edit1][aslano]\n"
            "[asland_text]content\n"
            "[aslano]\n"
            "\n"
            "[asland_edit2][aslano]\n"
            "[asland_text]more content\n"
            "[aslano]"
        )

        # Both should have the same structure (ignoring whitespace in content)
        assert result_inline == {
            "_default": None,
            "edit1": {
                "text": "content",
            },
            "edit2": {
                "text": "more content",
            },
        }

        assert result_multiline == {
            "_default": None,
            "edit1": {
                "text": "content\n",
            },
            "edit2": {
                "text": "more content\n",
            },
        }

    def test_max_object_depth_1_second_aslano_should_close_instead_of_nesting(self):
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
            'maxObjectDepth': 1,
        })

        # Attempting to nest more than 1 level deep
        result = parser.parse(
            "[asland_a][aslano][asland_b][aslano][asland_c]value[aslano][aslano]"
        )

        # With maxObjectDepth: 1:
        # - [aslano] after a: depth 0, 'a' is empty -> creates object for a (depth becomes 1)
        # - [asland_b]: sets current key to 'b' inside 'a' (Python creates empty string entry)
        # - [aslano] after b: depth 1 >= maxDepth -> always close (depth becomes 0)
        # - [asland_c]: sets key to 'c' at top level
        # - value: c = 'value'
        # - [aslano] after c/value: depth 0, follows normal logic (close noop at root)
        # - final [aslano]: same, noop
        # Note: Python parser creates empty string entries for data delimiters
        assert result == {
            "_default": None,
            "a": {"b": ""},
            "c": "value",
        }

    def test_max_object_depth_1_works_with_arrays_inside_objects(self):
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
            'maxObjectDepth': 1,
        })

        result = parser.parse(
            "[asland_edit][aslano][asland_items][aslana][asland]item1[asland]item2[aslano]"
        )

        # Array nesting should still work, only object depth is limited
        assert result == {
            "_default": None,
            "edit": {
                "items": ["item1", "item2"],
            },
        }

    def test_max_object_depth_0_no_objects_allowed(self):
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
            'maxObjectDepth': 0,
        })

        result = parser.parse(
            "[asland_edit][aslano][asland_text]content[aslano]"
        )

        # With maxObjectDepth: 0, [aslano] always tries to close (depth 0 >= 0)
        # At depth 0, closing is a no-op (nothing to pop from stack)
        # So [aslano] is effectively ignored
        # Note: Python parser creates empty string entries for data delimiters
        assert result == {
            "_default": None,
            "edit": "",
            "text": "content",
        }

    def test_max_object_depth_2_allows_two_levels_of_nesting(self):
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
            'maxObjectDepth': 2,
        })

        result = parser.parse(
            "[asland_a][aslano][asland_b][aslano][asland_c]value[aslano][aslano]"
        )

        # With maxObjectDepth: 2:
        # - [aslano] after a: depth 0 < 2, 'a' is empty -> creates object (depth 1)
        # - [aslano] after b: depth 1 < 2, 'b' is empty -> creates nested object (depth 2)
        # - [aslano] after c/value: depth 2 >= 2 -> closes (depth 1)
        # - final [aslano]: depth 1 < 2, but at this point we've returned to parent with non-empty content
        #   Following normal logic, this should close (depth 0)
        assert result == {
            "_default": None,
            "a": {
                "b": {
                    "c": "value",
                },
            },
        }

    def test_max_object_depth_none_default_behavior_unlimited_depth(self):
        parser = ASLANParser({
            'collapseObjectStartWhitespace': True,
            # maxObjectDepth not set - should behave as before (None means unlimited)
        })

        result = parser.parse(
            "[asland_a][aslano][asland_b][aslano][asland_c][aslano][asland_d]value"
        )

        # Without maxObjectDepth, deep nesting should work as before
        assert result == {
            "_default": None,
            "a": {
                "b": {
                    "c": {
                        "d": "value",
                    },
                },
            },
        }
