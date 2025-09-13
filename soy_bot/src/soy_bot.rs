use rust_sc2::prelude::*;
use std::collections::{HashMap, HashSet};

#[bot]
#[derive(Default)]
pub struct SoyBot {
    pub base_indices: HashMap<u64, usize>, // (base tag, expansion index)
    pub assigned: HashMap<u64, HashSet<u64>>, // (mineral, workers)
    pub free_workers: HashSet<u64>,        // tags of workers which aren't assigned to any work
    pub harvesters: HashMap<u64, (u64, u64)>, // (worker, (target mineral, nearest townhall))
}

impl SoyBot {
    pub fn assign_roles(&mut self) {
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
    }
    pub fn execute_micro(&mut self) {
        let (gather_ability, return_ability) = match self.race {
            Race::Terran => (AbilityId::HarvestGatherSCV, AbilityId::HarvestReturnSCV),
            Race::Zerg => (AbilityId::HarvestGatherDrone, AbilityId::HarvestReturnDrone),
            Race::Protoss => (AbilityId::HarvestGatherProbe, AbilityId::HarvestReturnProbe),
            _ => unreachable!(),
        };
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
                        if ability == gather_ability && t == *mineral_tag =>
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
                        if ability == return_ability && t == *base_tag =>
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
