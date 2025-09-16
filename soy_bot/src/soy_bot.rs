use rust_sc2::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

#[bot]
#[derive(Default)]
pub struct SoyBot {
    // (Base Tag, Expansion Index)
    pub base_indices: HashMap<u64, usize>,
    // (Mineral Patch, Workers)
    pub assigned: HashMap<u64, HashSet<u64>>,
    // tags of workers which aren't assigned to any work
    pub free_workers: HashSet<u64>,
    // (worker, (target mineral, nearest townhall))
    pub harvesters: HashMap<u64, (u64, u64)>,
    // Gather ability for workers
    pub gather_ability: Option<AbilityId>,
    // Return ability for workers
    pub return_ability: Option<AbilityId>,
    // A list of workers who are building things
    pub builders: HashSet<u64>,
    // Orders for what to train next
    pub train_queue: VecDeque<UnitTypeId>,
    // Orders for what to build next
    pub build_queue: VecDeque<UnitTypeId>,
}

impl SoyBot {
    pub fn tactician(&mut self) {
        //supply.left
        if self.supply_left < 2 {
            self.build_queue.push_front(UnitTypeId::SupplyDepot);
            println!("[TACTICIAN]\tSupply Depot added to build queue");
        }
        if self.supply_workers < 200 && !self.train_queue.contains(&UnitTypeId::SCV) {
            self.train_queue.push_back(UnitTypeId::SCV);
            println!("[TACTICIAN]\tSCV added to train Queue");
        }
    }

    pub fn train_units(&mut self) {
        // get first element if queue is not empty
        if let Some(unit) = self.train_queue.front() {
            match unit {
                UnitTypeId::SCV => {
                    if self.can_afford(UnitTypeId::SCV, true)
                        && let Some(cc) = self
                            .units
                            .my
                            .townhalls
                            .iter()
                            .find(|u| u.is_ready() && u.is_almost_idle())
                    {
                        cc.train(UnitTypeId::SCV, false);
                        self.subtract_resources(UnitTypeId::SCV, true);
                        self.train_queue.pop_front();
                        println!("[TRAIN UNITS]\tTraining SCV");
                    }
                }
                _ => {}
            }
        }
    }
    pub fn build(&mut self) {
        let main_base = self.start_location.towards(self.game_info.map_center, 8.0);
        if let Some(building) = self.build_queue.front() {
            match building {
                UnitTypeId::SupplyDepot => {
                    if self.can_afford(UnitTypeId::SupplyDepot, false) {
                        if let Some(location) = self.find_placement(
                            UnitTypeId::SupplyDepot,
                            main_base,
                            Default::default(),
                        ) {
                            // take an owned copy of a harvester tag, then remove the assignment
                            let last_harvester_tag = self
                                .harvesters
                                .keys()
                                .copied()
                                .last()
                                .expect("No workers found while trying to build supply depot:(");
                            // remove the harvester assignment before borrowing self for the unit reference
                            self.harvesters.remove(&last_harvester_tag);
                            let actual: &Unit = self
                                .units
                                .my
                                .workers
                                .iter()
                                .find(|&worker| worker.tag() == last_harvester_tag)
                                .expect("Harvester not found");
                            actual.build(UnitTypeId::SupplyDepot, location, false);
                            self.subtract_resources(UnitTypeId::SupplyDepot, false);
                            return;
                        }
                    }
                }
                _ => {}
            }
        }
    }

    pub fn get_worker_abilities(&mut self) {
        let (gather, ret) = match self.race {
            Race::Terran => (AbilityId::HarvestGatherSCV, AbilityId::HarvestReturnSCV),
            Race::Zerg => (AbilityId::HarvestGatherDrone, AbilityId::HarvestReturnDrone),
            Race::Protoss => (AbilityId::HarvestGatherProbe, AbilityId::HarvestReturnProbe),
            _ => unreachable!(),
        };
        self.gather_ability = Some(gather);
        self.return_ability = Some(ret);
    }
    pub fn manage_workers(&mut self) {
        let mut to_harvest = vec![];
        // iterator of (mineral tag, nearest base tag)
        let mut harvest_targets = self.base_indices.iter().flat_map(|(b, i)| {
            self.expansions[*i]
                .minerals
                .iter()
                .map(|m| (m, 2 - self.assigned.get(m).map_or(0, |ws| ws.len())))
                .flat_map(move |(m, c)| vec![(*m, *b); c])
        });

        for w in &self.free_workers {
            if let Some(t) = harvest_targets.next() {
                to_harvest.push((*w, t));
            } else {
                break;
            }
        }

        for (w, t) in to_harvest {
            self.free_workers.remove(&w);
            self.harvesters.insert(w, t);
            self.assigned.entry(t.0).or_default().insert(w);
        }
        let mut mineral_moving = HashSet::new();

        for u in &self.units.my.workers {
            if let Some((mineral_tag, base_tag)) = self.harvesters.get(&u.tag()) {
                let is_collides = || {
                    let range = (u.radius() + u.distance_per_step()) * 2.0;
                    !self.assigned[mineral_tag].iter().all(|&w| {
                        w == u.tag()
                            || mineral_moving.contains(&w)
                            || u.is_further(range, &self.units.my.workers[w])
                    })
                };

                match u.orders().first().map(|ord| (ord.ability, ord.target)) {
                    // moving
                    Some((AbilityId::MoveMove, Target::Pos(current_target))) => {
                        let mineral = &self.units.mineral_fields[*mineral_tag];
                        let range = mineral.radius() + u.distance_per_step();
                        // moving towards mineral
                        if current_target.is_closer(range, mineral) {
                            // execute gather ability if close enough or colliding with other workers
                            if u.is_closer(u.radius() + range, mineral) || is_collides() {
                                u.smart(Target::Tag(mineral.tag()), false);
                                mineral_moving.insert(u.tag());
                            }
                            // otherwise keep moving
                            continue;
                        } else {
                            let base = &self.units.my.townhalls[*base_tag];
                            let range = base.radius() + u.distance_per_step();
                            // moving towards base
                            if current_target.is_closer(range, base) {
                                // execute return ability if close enough or colliding with other workers
                                if u.is_closer(u.radius() + range, base) || is_collides() {
                                    u.smart(Target::Tag(base.tag()), false);
                                    mineral_moving.insert(u.tag());
                                }
                                // otherwise keep moving
                                continue;
                            }
                        }
                    }
                    // gathering
                    Some((ability, Target::Tag(t)))
                        if ability
                            == self.gather_ability.expect("gather_ability is not assigned")
                            && t == *mineral_tag =>
                    {
                        let mineral = &self.units.mineral_fields[*mineral_tag];
                        // execute move ability if far away from mineral and not colliding with other workers
                        if u.is_further(
                            u.radius() + mineral.radius() + u.distance_per_step(),
                            mineral,
                        ) && !is_collides()
                        {
                            let base = &self.units.my.townhalls[*base_tag];
                            u.move_to(
                                Target::Pos(
                                    mineral
                                        .position()
                                        .towards(base.position(), mineral.radius()),
                                ),
                                false,
                            );
                        // otherwise keep gathering
                        } else {
                            mineral_moving.insert(u.tag());
                        }
                        continue;
                    }
                    // returning
                    Some((ability, Target::Tag(t)))
                        if ability
                            == self.return_ability.expect("return_ability is not assigned")
                            && t == *base_tag =>
                    {
                        let base = &self.units.my.townhalls[*base_tag];
                        // execute move ability if far away from base and not colliding with other workers
                        if u.is_further(u.radius() + base.radius() + u.distance_per_step(), base)
                            && !is_collides()
                        {
                            u.move_to(
                                Target::Pos(base.position().towards(u.position(), base.radius())),
                                false,
                            );
                        // otherwise keep returning
                        } else {
                            mineral_moving.insert(u.tag());
                        }
                        continue;
                    }
                    _ => {}
                }

                // execute default ability if worker is doing something it shouldn't do
                if u.is_carrying_resource() {
                    u.return_resource(false);
                } else {
                    u.gather(*mineral_tag, false);
                }
                mineral_moving.insert(u.tag());
            }
        }
    }
}
