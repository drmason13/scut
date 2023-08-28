# turns

* *turn*: the turn you have most recently finished (first turn = 0)
* *teammate_turn*: the turn your teammate has most recently finished (Option)
* *next_turn*: the turn you want to start next

* *enemy_turn*: the turn the enemy side has most recently finished (first turn = 0)
* *next_enemy_turn*: the turn the enemy side wants to start next

Not sure whether to include enemy turns in `TurnDetail``
```

    /// The latest turn the enemy side has uploaded belonging to their side
    /// 
    /// 0 if no turn has been uploaded
    pub enemy_turn: Turn,
    
```

## Examples
```
<Axis DM>
Local:
Axis 1
Axis DG 1

Remote:
Axis 1
Axis DG 1

turn = 1

<Axis DG>
Local:
Axis 1
Axis DG 1

Remote:
Axis 1
Axis DG 1

turn = 1

<Allies TG>
Local:

Remote:
Axis 1
Axis DG 1

turn = 1

<Axis DM>
Local:
Axis 1
Axis DM 1
Axis DG 1

Remote:
Axis 1
Axis DM 1
Axis DG 1
Allies 1

turn = 2

<Axis DM>
Local:
Axis 1
Axis DM 1
Axis DG 1

Remote:
Axis 1
Axis DM 1
Axis DG 1
Axis DG 1B
Allies 1
Axis 2
Axis DM 2
Axis DG 2
Allies 2
Axis 3
Axis DM 3
Axis DG 3
Axis DG 3B
Allies 3
Axis 4
Axis DM 4
Axis DG 4
Axis DG 4B
Allies 4

turn = 5
:download missing Axis Turn 4 saves (Axis DM 4, Axis DG 4, Axis DG 4B):
```