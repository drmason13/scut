# turn lifecycle

1. Turn Start

* goto 2 or 3

2. Player A uploads a save

* goto 2, 3, 4 or 5

3. Player A uploads a save with some part

* goto 2, 3 or 4 or 5

4. Player B uploads a save

* goto 2, 3, 4, 5 or 6

5. Player B uploads a save with some part

* goto 2, 3, 4, 5 or 6

6. Player (A or B) uploads Turn Start for other Side

* goto 1


**This can be simplified to**

1. Turn start

2. Player A or B upload a Save, possibly with a Part

* goto 2 or 3

3. Player A or B uploads Turn Start for other Side

## Determining what to turn upload a save as

Previously we have used a mutable config that increments the turn after certain upload actions, but this has proven brittle in the face of multi-part turns.

### Example

* Player A played part of a turn
* Player A uploaded - turn incremented without user input `:(`
* Player A played another part of the same turn
* Player A try to upload
* *wrong turn* "no teammate's save found for turn `<next turn>`"

An init sequence that uses the available turns in both remote and local storage could instead be the mechanism for incrementing the config's turn.

**Alternatively**

An init sequence that uses the available turns in both remote and local storage as the mechanism for incrementing the config's turn.

* scut init starts
1. list all local saves
2. list all remote saves
3. is there a turn start save for the opposing side that is 1 higher than (if allies) or equal to (if axis) your current turn?
* Y: increment the turn
* N: don't

1. get latest enemy turn start
2. get latest friendly turn start
3. get_turn_number(latest_enemy_turn, latest_friendly_turn) -> turn


# save decision algorithm

turn, player, side => Save

(first turn considerations)

.. ???? !? ... I've never been particularly clear on this! :D

## examples

In remote there are 6 (or more) saves:
```
Axis DG 1
Axis DM 1
Allies 1
Allies GM 1
Allies TG 1
Axis 2
```

In DM local there are 3 saves
```
autosave
Axis DG 1
Axis DM 1
```

DM runs scut download with a config set to turn 2 (and player DM and side Axis).

scut downloads Axis 2 and overwrites autosave

This is the simplest case, we'll call it "start save only"

Consider if instead, DM local has only these 2 saves:

```
autosave
Axis DM 1
```

scut downloads Axis 2 and Axis DG 1

This is case "start save and teammate's prev save"

# alternative approach

As GM keeps saying, just... download your team's saves from remote that aren't in local
and... upload your team's saves from local that aren't in remote

Simple!

