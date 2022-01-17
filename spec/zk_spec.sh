#!/bin/bash

tmp=$(mktemp -dp /tmp zk_spec_tests.XXXXXX)
# shellcheck disable=SC2064
trap "rm -r $tmp" EXIT
cd "$tmp" || exit 1

Describe 'zk'

    Mock zk
        if [ -z "$ZK_BIN" ]; then
            echo 'please set ZK_BIN'
            exit 1
        fi
        $ZK_BIN "$@"
    End
    
    Mock date
        case $1 in
            '--iso-8601=seconds')
                echo '2022-01-01T05:00:00-03:00' ;;
            '+%Y-%m-%d')
                echo '2022-01-01' ;;
            *)
                echo "<invalid input to date>" ;;
        esac
    End

    It 'creates a new metadatabase'
        When call zk init
        The output should include 'initalized new zk metadatabase at ./_zettel.json'
    End

    Parameters
      created modified
    End

    It "the metadatabase file has a meta record with $1"
        When call jq --raw-output ".meta.$1" _zettel.json
        The output should equal '2022-01-01T05:00:00-03:00'
    End

    It 'the metadatabase file has an empty zettels record'
        When call jq --raw-output '.zettels' _zettel.json
        The output should equal '{}'
    End

    Mock uuid
        echo '8918f638-7620-11ec-b116-00163e5e6c00'
    End

    Mock date
        case $1 in
            '--iso-8601=seconds')
                echo '2022-01-02T13:00:00-03:00' ;;
            '+%Y-%m-%d')
                echo '2022-01-02' ;;
            *)
                echo "<invalid input to date>" ;;
        esac
    End

    It 'creates a new zettel'
        When call zk new
        The output should equal "created a new zettel at 2022-01-02-my-note.md"
    End

    It "the metametadata modification date reflects the newly created zettel"
        When call jq --raw-output ".meta.modified" _zettel.json
        The output should equal '2022-01-02T13:00:00-03:00'
    End

    It 'the zettel has yaml frontmatter'
        When call cat 2022-01-02-my-note.md
        The output should equal '---
uuid: "8918f638-7620-11ec-b116-00163e5e6c00"
title: "my note"
---'
    End

    It 'adds a metadata entry for the new zettel'
        When call jq --raw-output '.zettels."8918f638-7620-11ec-b116-00163e5e6c00"' _zettel.json
        The output should equal '{
  "created": "2022-01-02T13:00:00-03:00",
  "modified": "2022-01-02T13:00:00-03:00",
  "path": "2022-01-02-my-note.md"
}'
    End

    Mock uuid
        echo 'c5a8e8c4-7625-11ec-9595-00163e5e6c00'
    End

    Mock date
        case $1 in
            '--iso-8601=seconds')
                echo '2022-01-03T16:20:00-03:00' ;;
            '+%Y-%m-%d')
                echo '2022-01-03' ;;
            *)
                echo "<invalid input to date>" ;;
        esac
    End

    It 'creates another zettel'
        When call zk new 'another note'
        The output should equal "created a new zettel at 2022-01-03-another-note.md"
    End

    It 'the metametadata modification date reflects the creation of the other zettel'
        When call jq --raw-output '.meta.modified' _zettel.json
        The output should equal '2022-01-03T16:20:00-03:00'
    End

    It 'the other zettel also has yaml frontmatter'
        When call cat 2022-01-03-another-note.md
        The output should equal '---
uuid: "c5a8e8c4-7625-11ec-9595-00163e5e6c00"
title: "another note"
---'
    End

    It 'also adds a metadata entry for the other zettel'
        When call jq --raw-output '.zettels."c5a8e8c4-7625-11ec-9595-00163e5e6c00"' _zettel.json
        The output should equal '{
  "created": "2022-01-03T16:20:00-03:00",
  "modified": "2022-01-03T16:20:00-03:00",
  "path": "2022-01-03-another-note.md"
}'
    End

    It 'can update a file that has moved'
        mv 2022-01-03-another-note.md 2022-01-05-cool-title.md
        When call zk update
        The output should equal 'updated
    2022-01-03-another-note.md -> 2022-01-05-cool-title.md
'
    End

    It 'tracks the update in the metadatabase file'
        When call jq --raw-output '.zettels."c5a8e8c4-7625-11ec-9595-00163e5e6c00".path' _zettel.json
        The output should equal '2022-01-05-cool-title.md'
    End

End
