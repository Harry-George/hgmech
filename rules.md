# Alpha Strike — Quick Start Rules (Introductory)

A developer reference distilled from *BattleTech: Alpha Strike Quick Start Rules* (2019-08).
Covers introductory-level **ground combat between BattleMechs (`BM`)** only. Fiction and
out-of-scope unit types (vehicles, infantry, aerospace) are omitted. All distances are in
**board inches**.

---

## 1. The Unit Card

Each unit is described by a card with the following stats:

| Field                | Meaning                                                                                                                                                         |
|----------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Name**             | Unit design name.                                                                                                                                               |
| **Type (TP)**        | Two-letter class code. Introductory rules use **`BM`** (BattleMech) only.                                                                                       |
| **Point Value (PV)** | Approximate battlefield strength; used for balancing forces.                                                                                                    |
| **Size (SZ)**        | Weight class: **1** Light, **2** Medium, **3** Heavy, **4** Assault. Used in physical-attack damage formulas.                                                   |
| **TMM**              | Target Movement Modifier — the to-hit penalty an attacker suffers for this unit's *standard* movement. Changes with movement mode, damage, and heat.            |
| **Move (MV)**        | Max inches movable per turn. A trailing letter denotes a special mode, e.g. `j` = jump (`10"j`).                                                                |
| **Role**             | Typical tactical role (flavor / force-building only).                                                                                                           |
| **Skill**            | Base Target Number for this unit's attacks. Regular ≈ 4; elite = 1 or 0.                                                                                        |
| **Damage (S/M/L)**   | Damage dealt at Short `(+0)`, Medium `(+2)`, Long `(+4)` range. A `0` or `—` means no attack possible at that bracket (a `0*` means *minimal damage* — see §6). |
| **Overheat (OV)**    | Extra damage the unit can add to an attack by taking equal heat.                                                                                                |
| **Heat Scale**       | Four boxes: `1 2 3 S`. The 4th box (`S`) = automatic shutdown.                                                                                                  |
| **Armor (A)**        | External armor points (white bubbles). Damaged first.                                                                                                           |
| **Structure (S)**    | Internal structure points (gray bubbles). Damaged after armor; all gone = destroyed.                                                                            |
| **Special**          | Special Abilities (see §10).                                                                                                                                    |
| **Critical Hits**    | Track marks for Engine / Fire Control / MP / Weapon / Ammo crits.                                                                                               |

---

## 2. Sequence of Play

A game is a series of **turns**, each with four phases in order:

### Step 1 — Initiative Phase

Both players roll **2D6**; re-roll ties. Higher total wins initiative. The initiative
**winner moves and fires *after* the loser** in the following phases (simulating better
tactical awareness).

### Step 2 — Movement Phase

Players **alternate** moving one unit at a time, **loser first**. With unequal unit counts,
the side with more units moves proportionally more units per alternation.

### Step 3 — Combat Phase

**Loser acts first**, but does **not** alternate — the acting player declares and resolves
**all** of their units' attacks, then the other player does the same. Each unit may make
**one** attack. Damage is calculated and recorded immediately but **does not take effect
until the End Phase** (so a unit destroyed this phase can still return fire).

### Step 4 — End Phase

Both players resolve simultaneously: apply all recorded damage and critical effects, remove
destroyed units, resolve heat changes, restart shutdown units. Then repeat from Step 1 until
a victory condition is met.

### Victory

Default: destroy all enemy units. Scenarios may define alternatives (breakthrough, capture,
hold-position, etc.).

---

## 3. Setup

1. Agree on a scenario.
2. Both sides roll **2D6**; higher = initiative winner, picks Force List first.
3. **Place terrain** by mutual agreement / alternation (hills, water, woods, buildings).
4. Initiative winner picks a **home edge**; opponent gets the opposite edge. Units normally
   enter from and exit through their own home edge.
5. **Starting positions:** units usually start *off-board* and enter turn 1, **or** (by
   agreement) deploy on-board within a **deployment zone** = the map area within **10" of the
   home edge**. Units may be placed with any facing.

---

## 4. Movement Phase

- **Base Move** = max inches per turn. A unit may move any direction, make multiple turns to
  steer around obstacles, and end facing any direction. It need not use its full Move.
- **Facing:** a 'Mech faces the direction its miniature's *feet* point. Facing only changes
  voluntarily during the Movement Phase and affects combat (front/rear hits, firing arcs).

### Movement Modes

Chosen per turn. Every unit has at least Standstill and Ground Move; some also have Jumping.

| Mode            | Trigger                                 | Effect on own attacks (attacker mod)       |
|-----------------|-----------------------------------------|--------------------------------------------|
| **Standstill**  | Moves < 1".                             | **−1** to-hit (easier to hit *and* shoot). |
| **Ground Move** | Moves ≥ 1", no jump. Default.           | **+0**.                                    |
| **Jumping**     | Has `j` in Move; ignores terrain costs. | **+2** to-hit.                             |

- **Jumping** always takes the shortest path: pick an endpoint within jump Move, land there
  with any facing. May jump *into* water but not *out of* it. Jumping ignores terrain
  movement costs and level-change limits.
- **Minimum Movement:** any mobile unit (Move > 0) may always move **2"** in any direction
  regardless of terrain cost (unless terrain is prohibited).

### Terrain & Movement Cost

| Terrain Type              | Movement Cost                                                  |
|---------------------------|----------------------------------------------------------------|
| Clear                     | 1"                                                             |
| Rough / Rubble            | +1"                                                            |
| Woods                     | +1"                                                            |
| Water                     | +1"                                                            |
| Level change (up or down) | +2" per 1" of elevation (**max 2" elevation per 1" traveled**) |

- Costs combine (e.g. changing elevation while in water).
- **Water** is assumed **1" deep** in the Quick Start rules; entering pays move cost + water
  cost + any level-change cost.
- **Level changes** steeper than 2" elevation per 1" horizontal travel are **prohibited**. If
  a unit lacks enough remaining Move to finish a climb, it stays at the previous level.

### Stacking

- A unit may pass through spaces occupied by **friendly** units.
- It may **not** pass through **enemy** units at the **same elevation** (different elevations,
  e.g. a jumping unit overhead, are OK).
- **No two units may end movement occupying the same space**, regardless of elevation.

---

## 5. Combat Phase — Weapon Attacks

Each unit may make one attack (weapon **or** physical). Sequence:

1. Verify Line of Sight
2. Verify Firing Arc
3. Determine Range
4. Make the Attack (compute Target Number)
5. Roll to Hit
6. Determine & Apply Damage
7. Roll for Critical Hits (if applicable)

### Step 1 — Line of Sight (LOS)

- LOS exists if the attacker can "see" the target from its vantage point.
- **Solid terrain** (hills, buildings): if **less than 1/3** of the target is visible, LOS is
  **blocked**.
- **Woods (non-solid):** blocks LOS only when the line passes through **≥ 6"** of woods.
  Woods that intervene but don't block impose a **+1** attack modifier.
- **Adjacent (base-to-base)** units always have LOS to each other.
- **Intervening units** are *not* terrain — they never block LOS or attacks.
- **Partial Cover:** if **more than 1/3 but less than 2/3** of the target is hidden by
  *blocking* (solid) terrain, LOS is not blocked but the target gains partial cover (**+1**
  attacker modifier). Woods do **not** grant partial cover.
- **Water:** a 'Mech standing in water gets partial cover from it, even if the attacker is
  higher and could otherwise see its legs.

### Step 2 — Firing Arc

Each unit has a field of fire based on type and facing. **If more than half the target's base
lies outside the attacker's firing arc, the attack cannot be made.**

### Step 3 — Range

Measure base-edge to base-edge.

| Distance           | Range Bracket | Range Modifier |
|--------------------|---------------|----------------|
| Up to 6"           | **Short**     | +0             |
| Over 6" up to 24"  | **Medium**    | +2             |
| Over 24" up to 42" | **Long**      | +4             |

- Use the attacker's S/M/L damage value for the bracket. A `0`/`—` value = no attack at that
  range.
- **Base-to-base** units cannot make *weapon* attacks against each other — only physical
  attacks.

### Step 4 — Target Number

```
Target Number = Skill + Range mod + Target movement mod + Attacker movement mod
              + Terrain mods + other mods
```

All modifiers are cumulative. The 2D6 roll must **equal or exceed** the final Target Number.

### Step 5 — Roll to Hit

Roll **2D6**. ≥ Target Number → hit.

### Step 6 — Determine & Apply Damage

**Attack direction (front/rear):** draw a straightedge from attacker-center to
target-center. If it enters through the target's **rear**, it's a rear hit; otherwise front.
Ties (hitting a corner) — target chooses.

**Damage amount** = attacker's damage value at the relevant range bracket.

- **+1 damage** for a rear hit.
- **Overheat:** add OV points used (declared before the roll) — see §8.
- **Heat special ability (`HT#/#/#`):** adds heat to the target in the End Phase (see §10).

**Minimal Damage (`0*`):** when attacking at a bracket with a `0*` value, roll **1D6** — on
**4+** deal 1 point of standard damage, otherwise 0. A minimal attack that deals 0 cannot
trigger critical-hit checks.

**Applying damage (Q&A order):**

1. Was the roll a **natural 12**? → roll once on the Critical Hits Table, then continue.
2. Armor remaining? → mark one armor bubble per damage point until damage or armor is
   exhausted.
3. Damage remaining after armor gone? → continue, else done.
4. Structure remaining? → mark one structure bubble per remaining damage point.
5. Damage still remaining after structure gone? → **unit destroyed**.
6. If structure was damaged (but unit survives) → roll once on the Critical Hits Table.

### Step 7 — Critical Hits

**Any hit that damages structure** triggers a **2D6** roll on the table below. Effects are
**permanent** and must be marked. If a rolled crit doesn't apply (e.g. a Weapon Hit on a unit
already at 0 damage), instead apply **+1 damage** (no further crit roll from that extra
damage).

#### Determining Critical Hits Table

| 2D6 | Effect                      |
|-----|-----------------------------|
| 2   | Ammo Hit                    |
| 3   | Engine Hit                  |
| 4   | Fire Control Hit            |
| 5   | No Critical Hit             |
| 6   | Weapon Hit                  |
| 7   | No Critical Hit             |
| 8   | MP Hit                      |
| 9   | Weapon Hit                  |
| 10  | No Critical Hit             |
| 11  | Fire Control Hit            |
| 12  | Engine Hit + Unit Destroyed |

> Note: the source prints "Engine Hit / Unit Destroyed" stacked on the 12 row; treat 12 as a
> destructive engine hit (functionally the unit is destroyed).

#### Critical Hit Effects

- **Ammo Hit:** unit **destroyed**, unless it has `CASE`/`CASEII`/`ENE`. With `CASE`: 1 extra
  damage instead (re-roll crit if that damages structure). With `CASEII`/`ENE`: treat as No
  Critical Hit.
- **Engine Hit:** unit generates **+1 heat whenever it fires** (no extra damage). A **second**
  Engine Hit destroys the unit.
- **Fire Control Hit:** **+2** cumulative to-hit modifier on all future *weapon* attacks (not
  physical).
- **MP Hit:** lose **half** current Move and TMM (round normally; min Move loss 2", min TMM
  loss 1). Reduced to Move ≤ 0 → immobile.
- **Weapon Hit:** all damage values **−1** (min 0). Does not affect physical attack values.
- **No Critical Hit:** no effect.
- **Unit Destroyed:** removed from play.

---

## 6. Physical Attacks

Similar to weapon attacks but no range step. A unit **cannot** make a physical attack the same
turn it made a weapon attack, and may make only **one** physical attack per turn.

Sequence: (1) determine type → (2) make attack → (3) roll to hit → (4) damage → (5) crits.

### Types

| Type                                 | Requirement                                                                                                        | Damage                |
|--------------------------------------|--------------------------------------------------------------------------------------------------------------------|-----------------------|
| **Standard** (punch/kick)            | Within **1"** of target, target in firing arc.                                                                     | = attacker **Size**.  |
| **Melee**                            | Unit has `MEL`; within **2"** of target, target in arc. Replaces Standard for MEL units.                           | = Size **+1**.        |
| **Charge** (special)                 | Uses ground move to ram; must end base-to-base; target must have finished moving.                                  | See formula below.    |
| **Death From Above / DFA** (special) | Needs jumping Move sufficient to reach target; must end base-to-base; target finished moving; target not airborne. | Charge damage **+1**. |

Only **one** special physical attack (Charge or DFA) may target a given unit per turn.

### Target Number

Base = Skill, modified by type and target movement/terrain mods (cumulative).

- Charge / DFA: **+1** attack modifier. Standard / Melee: **+0**.
- Fire Control Hit modifier does **not** apply to physical attacks.
- Shutdown targets give **no** movement modifier for physical attacks.

### Charge / DFA Damage Formulas

```
Charge Damage     = floor-or-round( Inches Charged × Size ÷ 8 )   [round fractions normally]
Death From Above  = Charge Damage + 1
```

- **Charge — damage to attacker:** if target Size ≥ 3, attacker takes **1** damage.
- **DFA — damage to attacker:** on success, attacker takes damage = **its own Size**. On
  failure, attacker takes **1** damage (**+1** more if attacker Size ≥ 3).
- A successful DFA always forces **1 crit roll** on the target (even with no structure
  damage); if it *also* damaged structure, roll an **additional** crit.
- Charge/DFA damage to the target does **not** count as the target's own attack — the target
  still attacks normally in its Combat Phase.

---

## 7. End Phase

- Apply all damage and crit effects recorded during the Combat Phase; remove destroyed units.
- Resolve heat changes (§8) and restart shutdown units.

---

## 8. Heat & Overheating

### Using Overheat Value

- Declare OV use (and how many points, 0…OV) **before** the attack roll.
- On a hit, add that many points of damage at **Short or Medium** range (also **Long** only if
  the unit has `OVL`).
- Each OV point used adds **1 heat** to the heat scale in the End Phase (**−1** if the unit is
  in water).
- Cannot overheat beyond what the heat scale allows.
- `HT#/#/#` attacks and **physical attacks** may **not** be augmented by overheat.

### Heat Scale Effects

Current heat level applies starting the **turn after** it's marked (heat changes only at End
Phase). While at a given heat level:

- **+heat level** to all weapon-attack Target Numbers.
- **−(2 × heat level) inches** of **ground** Move.
- At heat level **≥ 2**: **−1 TMM**.
- Jumping Move and jumping TMM are **not** affected by heat.

### Shutdown

- Heat level **4** = the **`S`** box = automatic **shutdown**: the unit cannot move or attack
  the **following** turn.
- A shutdown unit is **immobile** → target movement modifier **−4**.

### Cooling Down (End Phase)

- A unit that **used overheat** this turn does **not** cool at all (heat rises as above).
- A unit that **made a weapon attack but didn't overheat** (and isn't in deep water): heat
  **unchanged**.
- A unit **in water using only 1 overheat**: heat unchanged.
- Heat **decreases** in the End Phase only when:
    - A **shutdown** unit → drops to **0** and restarts.
    - A unit that made **no weapon attack** → drops to **0**.
    - A unit entering water **≥ 2"** deep → **−1** heat (only if it used no overheat).

### Heat (`HT#/#/#`) Special Ability targeting

- Applies heat to the *target's* heat scale in the End Phase of a successful hit.
- A unit may receive at most **2 points** of heat from `HT` attacks per turn; extra is lost.
- If the target doesn't use a heat scale, the heat becomes normal attack damage instead.

---

## 9. Quick Reference — Attack Modifiers Table

**Base Target Number = unit Skill.** All modifiers cumulative.

**Attacker Movement**
| Attacker did | Modifier |
|--------------|----------|
| Standstill | −1 |
| Ground Movement | +0 |
| Jumping Movement | +2 |

**Target Movement**
| Target did | Modifier |
|------------|----------|
| Standstill | +0 |
| Ground Movement | +TMM |
| Jumping Movement | +TMM +1 |
| Immobile / Shutdown | −4 |

**Other**
| Condition | Modifier |
|-----------|----------|
| Intervening / Occupied Woods | +1 |
| Partial Cover | +1 |
| Attacker has Heat Level > 0 | +Heat level |
| Attacker has a Fire Control Hit | +2 per hit |

**Range** (weapon attacks only)
| Range | Modifier |
|-------|----------|
| Short (≤6") | +0 |
| Medium (>6"–24") | +2 |
| Long (>24"–42") | +4 |

**Physical Attack Type**
| Type | Modifier |
|------|----------|
| Charge / Death From Above | +1 |
| Standard / Melee | +0 |

> Heat-level and Fire-Control modifiers do **not** apply to physical attacks.

---

## 10. Special Abilities (Introductory subset)

If a special ability contradicts a basic rule, the ability wins. Abilities not listed here
have no effect at introductory level.

| Ability                 | Effect                                                                                                                                                        |
|-------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **CASE**                | Survives Ammo Hit crits, but takes 1 extra damage (re-roll crit if it damages structure).                                                                     |
| **CASEII**              | Ignores Ammo Hit crits entirely (treat as No Critical Hit).                                                                                                   |
| **ENE** (Energy)        | No ammo to explode — ignores Ammo Hit crits.                                                                                                                  |
| **HT#/#/#** (Heat)      | Adds the listed heat to the target's heat scale (S/M/L) in the End Phase of a successful hit. Max 2 heat/target/turn from HT. Can't be augmented by overheat. |
| **MEL** (Melee)         | Has a melee weapon: +1 physical attack damage on a Melee attack; must use Melee instead of Standard.                                                          |
| **OVL** (Overheat Long) | Overheat damage bonus also applies at Long range (not just S/M).                                                                                              |

---

## 11. Reference: Introductory Units (from scenarios)

Skills are scenario-specific; the unit's intrinsic Size/Move/Damage come from its card.
Designs appearing in the booklet's scenarios:

- BattleMechs: Marauder MAD-3R, Warhammer WHM-6D/6R, Hatchetman HCT-3F, Firestarter FS9-H,
  Banshee BNC-3E, Trebuchet TBT-5N, Blackjack BJ-1, Cataphract CTF-1X/3L, Orion ON1-K,
  Vindicator VND-1R, Hermes II HER-2M, Vulcan VL-2T, Commando COM-2D, Catapult CPLT-C1,
  Zeus ZEU-6S, Clint CLNT-2-3T/2-4T, Black Knight BL-7-KNT, Centurion CN9-AL, Javelin JVN-10N.

### Example values mentioned in the rules text

- **Stalker STK-3F:** Damage 3/4/2, OV 3, no OVL. With OV 3: up to 6 dmg Short, 7 Medium, but
  still 2 at Long (no OVL).
- **Rifleman RFL-3N:** 4 Armor, 5 Structure (undamaged).

---

## 12. Formations & Special Pilot Abilities (SPAs)

Inner Sphere 'Mechs organize in **Lances of 4**. A formation grants SPAs; if a formation drops
below **3 surviving units**, it loses its formation bonus and SPAs.

| Lance                       | Bonus                                      | SPA granted                                                                                                               |
|-----------------------------|--------------------------------------------|---------------------------------------------------------------------------------------------------------------------------|
| **Battle Lance**            | Line troops; close range, heavy firepower. | **6× Lucky(1)** — after a failed attack roll, reroll once (each Lucky use is single-use). One reroll per attack max.      |
| **Fire Lance**              | Long-range fire support.                   | At each turn start, up to **2** units gain **Sniper**: Long range modifier → **+2** (from +4), Medium → **+1** (from +2). |
| **Cavalry / Striker Lance** | Fast flankers.                             | At setup, **3** units gain **Speed Demon**: +2" Move per turn (does **not** change TMM).                                  |

---

## Appendix — Scenario Setups

**Deployment zone** for on-board start = within 10" of home edge; typical map ~24" deep.

- **Green — "Training Day":** 2 v 2. Terrain: 1 Woods + 1 Water near center, ≥4" apart.
  Attacker: Marauder MAD-3R (Skill 3), Firestarter FS9-H (3). Defender: Warhammer WHM-6D (3),
  Hatchetman HCT-3F (3). **Win:** destroy the enemy's Size-3 'Mech (Marauder/Warhammer).
- **Veteran — "Noble Feud":** Lance v Lance (4 each), both Marauder MAD-3R, Warhammer WHM-6D,
  Hatchetman HCT-3F (Skill 3), Firestarter FS9-H (3). Minimal terrain. **Win:** destroy 2
  enemy units. *Variation "Hold The Line":* attacker gets a 2nd Marauder + Firestarter; ≥4
  terrain pieces; defender picks home edge; defender must destroy 4 attacker units before
  attacker destroys all 4 defenders.
- **Elite — "Factory Battle":** multi-lance forces; a **Factory** building = immobile unit
  with **9 Armor / 1 Structure**, destructible. A unit in LOS between attacker and factory may
  shield it: on a 2D6 roll of **7+** the unit takes the damage instead. **Objectives:** *Win
  the Field* (destroy half the enemy before losing half your own); *Salt the Earth* (destroy
  the factory). Win the field without the opponent salting the earth = victory; if salted =
  draw; neither = loss.

---

*Source: BattleTech Alpha Strike Quick Start Rules, © The Topps Company. This file is an
internal development reference only.*
