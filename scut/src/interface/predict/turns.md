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

turn = None
next_turn = 1
enemy_turn = None
next_enemy_turn = 1
current_enemy_turn = None

<Axis DG>
Local:
Axis 1
Axis DG 1

Remote:
Axis 1
Axis DG 1

turn = 1
next_turn = 2
enemy_turn = None
next_enemy_turn = 1
current_enemy_turn = None

<Allies TG>
Local:

Remote:
Axis 1
Axis DG 1

turn = None
next_turn = 1
enemy_turn = 1
next_enemy_turn = 2
current_enemy_turn = 1
```