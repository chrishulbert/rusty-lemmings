// This figures out which levels are applicable for which game+skill.
// This is a big fat workaround for the fact that the levels' relationship to the skill was hardcoded in the original game.

use std::collections::HashMap;

use lemmings::models::*;

// Lemmings 1 is a bit lousy: there are 30 levels per each of 4 difficulties (120) but only 80 level files - many are re-used with different skill numbers and names.
// Here i'm just matching them by the names in the level files, so it'll skip some, but it's only skipping the renamed duplicate ones so no big loss.
const LEMMINGS_1_FUN: &str = "
Just dig!
Only floaters can survive this
Tailor-made for blockers
Now use miners and climbers
You need bashers this time
A task for blockers and bombers
Builders will help you here
We all fall down
A Beast of a level
";

const LEMMINGS_1_TRICKY: &str = "
This should be a doddle!
We all fall down
MENACING !!
One way digging to freedom
All the 6`s ........
Turn around young lemmings!
From The Boundary Line
Tightrope City
Cascade
I have a cunning plan
The Island of the Wicker people
Lost something?
Rainbow Island
The Crankshaft
";

const LEMMINGS_1_TAXING: &str = "
If at first you don`t succeed..
Watch out, there`s traps about
Heaven can wait (we hope!!!!)
Lend a helping hand....
The Prison!
Compression Method 1
Every Lemming for himself!!!
The Art Gallery
Perseverance
Izzie Wizzie lemmings get busy
The ascending pillar scenario
Livin` On The Edge
Upsidedown World
Hunt the Nessy....
What an AWESOME level
Mary Poppins` land
X marks the spot
Tribute to M.C.Escher
Bomboozal
Walk the web rope
Feel the heat!
Come on over to my place
King of the castle
Take a running jump.....
Follow the leader...
Triple Trouble
Call in the bomb squad
POOR WEE CREATURES!
How do I dig up the way?
";

const LEMMINGS_1_MAYHEM: &str = "
Steel Works
The Boiler Room
It`s hero time!
The Crossroads
Down, along, up. In that order
One way or another
Poles Apart
Last one out is a rotten egg!
Curse of the Pharaohs
Pillars of Hercules
The Far Side
The Great Lemming Caper
Pea Soup
The Fast Food Kitchen...
Just a Minute...
Stepping Stones
And then there were four....
Time to get up!
With a twist of lemming please
A BeastII of a level
Going up.......
All or Nothing
Have a nice day!
The Steel Mines of Kessel
Just a Minute (Part Two)
Mind the step.....
Save Me
Rendezvous at the Mountain
";

// Lemmings 2, unlike 1, is good: All levels are unique.

const OHNOMORE_TAME: &str = "
Down And Out Lemmings
Rent-a-Lemming
Undercover Lemming
Downwardly Mobile Lemmings
Snuggle up to a Lemming
Intsy-Wintsy...Lemming?
Who`s That Lemming
Dangerzone
And now this...
New Lemmings On The Block
With Compliments
Citizen Lemming
Thunder-Lemmings are go!
Get a little extra help
Not just a pretty Lemming
Gone With The Lemming
Honey, I Saved The Lemmings
Lemmings For Presidents!
Lemming Productions Present...
Custom built for Lemmings
";

const OHNOMORE_CRAZY: &str = "
Quote: \"That`s a good level\"
Dolly Dimple
Many Lemmings make level work
Lemming Express
24 hour Lemathon
The Stack
And now, the end is near...
KEEP ON TRUCKING
On the Antarctic Coast
ROCKY VI
No Problemming!
Lemming Friendly
It`s a trade off
Time waits for no Lemming
Worra load of old blocks!
Across The Gap
DIGGING FOR VICTORY
NO PROBLEM
DON`T PANIC
Ice Ice Lemming
";

const OHNOMORE_WILD: &str = "
PoP YoR ToP!!!
Lemming Hotel
Lemming Rhythms
Meeting Adjourned
Lemming Head
Just A Quicky
You Take the High Road
It`s a tight fit!
Ice Station Lemming
Higgledy Piggledy
Mutiny On The Bounty
SNOW JOKE
ONWARD AND UPWARD
ICE SPY
THE SILENCE OF THE LEMMINGS
Take care, Sweetie
The Chain with no name
Dr Lemminggood
Lemmingdelica
Got anything....Lemmingy???
";

const OHNOMORE_WICKED: &str = "
LeMming ToMato KetchUp fAcilitY
Inroducing SUPERLEMMING
This Corrosion
Oh No! It`s the 4TH DIMENSION!
Chill out!
PoP TiL YoU DrOp!
Last Lemming To Lemmingcentral
A TOWERING PROBLEM
How on Earth?
Temple of Love
ROCKY ROAD
Suicidal Tendencies
Almost Nearly Virtual Reality
The Lemming Learning Curve
SPAM,SPAM,SPAM,EGG AND LEMMING
Five Alive
Down the tube
LoTs moRe wHeRe TheY caMe fRom
Up, Down or Round and Round
The Lemming Funhouse
";

const OHNOMORE_HAVOC: &str = "
Tubular Lemmings
Be more than just a number
It`s the price you have to pay
The race against cliches
There`s madness in the method
Now get out of that!
Creature Discomforts
Lemming about town
AAAAAARRRRRRGGGGGGHHHHHH!!!!!!
Flow Control
Welcome to the party, pal!
It`s all a matter of timing
HIGHLAND FLING
Synchronised Lemming
Have an ice day
Scaling the Heights
Where Lemmings Dare
Lemmings in a situation
Looks a Bit Nippy Out There
LOoK BeFoRe YoU LeAp!
";

fn names_per_game_and_skill(game_id: &str, skill: isize) -> &'static str {
    match (game_id, skill) {
        ("lemmings", 0) => LEMMINGS_1_FUN,
        ("lemmings", 1) => LEMMINGS_1_TRICKY,
        ("lemmings", 2) => LEMMINGS_1_TAXING,
        ("lemmings", 3) => LEMMINGS_1_MAYHEM,
        ("ohnomore", 0) => OHNOMORE_TAME,
        ("ohnomore", 1) => OHNOMORE_CRAZY,
        ("ohnomore", 2) => OHNOMORE_WILD,
        ("ohnomore", 3) => OHNOMORE_WICKED,
        ("ohnomore", 4) => OHNOMORE_HAVOC,
        _ => ""
    }
}

pub fn levels_per_game_and_skill(game_id: &str, skill: isize, level_map: &HashMap<i32, Level>) -> Vec<Level> {
    let mut levels: Vec<Level> = Vec::new();
    let names = names_per_game_and_skill(game_id, skill);
    for level_name in names.split("\n") {
        for (_, level) in level_map {
            if level.name == level_name {
                levels.push(level.clone());
                break;
            }
        }
    }
    return levels;
}
 