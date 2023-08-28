use std::process::Command;
use clap::Parser;

#[derive(Default, Debug)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn from_str(rpos: &str) -> Self {
        let rparts = rpos.split(' ')
            .skip(2)
            .take(1)
            .collect::<String>();

        let parts = rparts.split('x').collect::<Vec<&str>>();

        let x = parts[0].parse::<usize>().unwrap();
        let y = parts[1].parse::<usize>().unwrap();

        Self {
            x,
            y
        }
    }
}

#[derive(Default, Debug)]
struct Resolution {
    x: usize,
    y: usize,
    freq: f32,
}

impl Resolution {
    fn display(&self) -> String {
        format!("{}x{}@{}", self.x, self.y, self.freq)
    }

    fn from_str(rres: &str) -> Self {
        let res_str = rres.split(' ').take(1).collect::<String>();
        let parts = res_str.split('@').take(2).collect::<Vec<&str>>();

        let res_str = parts[0];
        let freq_str = parts[1];

        let parts = res_str.split('x').collect::<Vec<&str>>();

        let x = parts[0][1..].parse::<usize>().unwrap();
        let y = parts[1].parse::<usize>().unwrap();
        let freq = freq_str.parse::<f32>().unwrap();

        Self {
            x,
            y,
            freq
        }
    }
}

#[derive(Default, Debug)]
struct Monitor {
    ctlname: String,
    res: Resolution,
    id: usize,
    pos: Position,
}

impl Monitor {
    fn from_str(rmon: &str) -> Self {
        let ctlname = rmon.split(' ').skip(1).take(1).collect::<String>();
        let id = rmon.split(' ')
            .skip(3)
            .take(1)
            .collect::<String>()
            .split(')')
            .take(1)
            .collect::<String>();

        let id = id.parse::<usize>().unwrap();
        
        let res = Resolution::from_str(&rmon.split('\n').skip(1).take(1).collect::<String>());
        let pos = Position::from_str(&rmon.split('\n').skip(1).take(1).collect::<String>());

        Self {
            res,
            id,
            ctlname,
            pos
        }
    }
}

#[derive(Debug)]
struct Setup {
    monitors: Vec<Monitor>,
}

impl Setup {
    fn new() -> Self {
        let output = Command::new("hyprctl")
            .arg("monitors")
            .output()
            .expect("Couldn't execute hyprctl");

        let result = String::from_utf8_lossy(&output.stdout).to_string();

        let rmons = result.split("\n\n")
            .take_while(|x| x != &"\n") // Removes the trailing \n
            .map(Monitor::from_str)
            .collect::<Vec<Monitor>>();

        Setup { monitors: rmons }
    }
    
    fn move_monitor(&self, mon_name: &str, direction: ThrowDirection, tgt_name: &str) {
        if let Some(monitor_ref) = self.get_monitor_from_id_or_name(mon_name) {
            if let Some(tgt_mon_ref) = self.get_monitor_from_id_or_name(tgt_name) {
                match direction {
                    ThrowDirection::Left => Self::move_x_left_of_y(monitor_ref, tgt_mon_ref),
                    ThrowDirection::Right => Self::move_x_left_of_y(tgt_mon_ref, monitor_ref),
                    ThrowDirection::Above => Self::move_x_above_y(monitor_ref, tgt_mon_ref),
                    ThrowDirection::Under => Self::move_x_above_y(tgt_mon_ref, monitor_ref),
                }
            }

        }
    }

    fn move_x_left_of_y(monitor_ref: &Monitor, tgt_mon_ref: &Monitor) {
        if tgt_mon_ref.pos.x == 0 {
            Command::new("hyprctl")
                .arg("keyword")
                .arg("monitor")
                .arg(format!("{},{},0x{},1", monitor_ref.ctlname, monitor_ref.res.display(), tgt_mon_ref.pos.y))
                .output()
                .unwrap_or_else(|_| panic!("Moving {} to 0x{} failed", monitor_ref.ctlname, tgt_mon_ref.pos.y));
            Command::new("hyprctl")
                .arg("keyword")
                .arg("monitor")
                .arg(format!("{},{},{}x{},1", tgt_mon_ref.ctlname, tgt_mon_ref.res.display(), monitor_ref.res.x, tgt_mon_ref.pos.y))
                .output()
                .unwrap_or_else(|_| panic!("Moving {} to {}x{} failed", tgt_mon_ref.ctlname, monitor_ref.res.x, tgt_mon_ref.pos.y));
        } else {
            let new_pos_x = tgt_mon_ref.pos.x - tgt_mon_ref.res.x;
            Command::new("hyprctl")
                .arg("keyword")
                .arg("monitor")
                .arg(format!("{},{},{}x{},1", monitor_ref.ctlname, monitor_ref.res.display(), new_pos_x, tgt_mon_ref.pos.y))
                .output()
                .unwrap_or_else(|_| panic!("Moving {} to {}x{} failed", monitor_ref.ctlname, new_pos_x, tgt_mon_ref.pos.y));
        }
    }

    fn move_x_above_y(monitor_ref: &Monitor, tgt_mon_ref: &Monitor) {
        Command::new("hyprctl")
            .arg("keyword")
            .arg("monitor")
            .arg(format!("{},{},{}x{},1", monitor_ref.ctlname, monitor_ref.res.display(), tgt_mon_ref.pos.x, tgt_mon_ref.res.y))
            .output()
            .unwrap_or_else(|_| panic!("Moving {} to {}x{} failed", monitor_ref.ctlname, tgt_mon_ref.pos.x, tgt_mon_ref.res.y));
    }

    fn get_monitor_from_name(&self, mon_name: &str) -> Option<&Monitor> {
        for item in &self.monitors {
            if item.ctlname == mon_name {
                return Some(item)
            }
        }
        None
    }

    fn get_monitor_from_id(&self, rid: &str) -> Option<&Monitor> {
        if let Ok(id) = rid.parse::<usize>() {
            for item in &self.monitors {
                if item.id == id {
                    return Some(item)
                }
            }
            None
        } else {
            None
        }
    }

    fn get_monitor_from_id_or_name(&self, name_or_id: &str) -> Option<&Monitor> {
        if let Some(mon) = self.get_monitor_from_name(name_or_id) {
            Some(mon)
        } else if let Some(mon) = self.get_monitor_from_id(name_or_id) {
            Some(mon)
        } else {
            None
        }
    }

    fn disable_monitor(&self, name_or_id: &str) {
        if let Some(mon) = self.get_monitor_from_id_or_name(name_or_id) {
            Command::new("hyprctl")
                .arg("keyword")
                .arg("monitor")
                .arg(format!("{},disable", mon.ctlname))
                .output()
                .unwrap_or_else(|_| panic!("Could not disable monitor {}", mon.ctlname));
        } else {
            println!("Could not find monitor: {}", name_or_id);
        }
    }


    fn enable_monitor(&self, name_or_id: &str) {
        Command::new("hyprctl")
            .arg("keyword")
            .arg("monitor")
            .arg(format!("{},preferred,auto,1", name_or_id))
            .output()
            .unwrap_or_else(|_| panic!("Could not enable monitor {}", name_or_id));
    }

    fn only_monitor(&self, name_or_id: &str) {
        if let Some(mon) = self.get_monitor_from_id_or_name(name_or_id) {
            let _ = self.monitors.iter().take_while(|x| x.id != mon.id).map(|x| self.disable_monitor(&x.ctlname)).count();
        } else {
            println!("Could not find monitor: {}", name_or_id);
        }
    }

}

#[derive(Parser, Debug)]
struct Config {
    monitor: Option<String>,

    #[arg(short, long)]
    disable: bool,

    #[arg(short, long)]
    enable: bool,

    #[arg(short, long)]
    only: bool,

    #[arg(short, long)]
    throw: Option<String>,

    target: Option<String>,

    #[arg(short, long)]
    verbose: bool,
}


#[derive(Debug)]
enum ThrowDirection {
    Left,
    Right,
    Above,
    Under,
}

impl ThrowDirection {
    fn from_str(r_action: &str) -> Option<Self> {
        let l_action = r_action.to_lowercase();

        match l_action.as_str() {
            "left" => Some(Self::Left),
            "right" => Some(Self::Right),
            "above" => Some(Self::Above),
            "under" => Some(Self::Under),
            _ => None,
        }
    }
}

fn main() {
    let setup = Setup::new();
    let args = Config::parse();

    if args.monitor.is_none() && !args.verbose{
        println!("{:#?}", setup);
    }

    if args.verbose {
        println!("{:?}", setup);
        println!("{:?}", args);
    }

    if let Some(mon_name) = args.monitor {

        if args.throw.is_some() && args.target.is_some() {
            if let Some(direction) = args.throw {
                if let Some(direction) = ThrowDirection::from_str(&direction) {
                    let target = args.target.unwrap();
                    if target != mon_name {
                        setup.move_monitor(&mon_name, direction, &target);
                    } else {
                        println!("Target name can't be equal to monitor name");
                    }
                } else {
                    println!("Invalid direction");
                }
            }
        } else if args.disable {
            setup.disable_monitor(&mon_name);
        } else if args.enable {
            setup.enable_monitor(&mon_name);
        } else if args.only {
            setup.only_monitor(&mon_name);
        }

    }
}
