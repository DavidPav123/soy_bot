use crate::soy_bot::SoyBot;
use rust_sc2::prelude::*;

impl Player for SoyBot {
    /// Returns settings used to connect bot to the game.
    fn get_player_settings(&'_ self) -> PlayerSettings<'_> {
        PlayerSettings::new(Race::Zerg)
            .with_name("Soy Bot")
            .raw_crop_to_playable_area(true)
    }

    /// Called once on first step (i.e on game start).
    fn on_start(&mut self) -> SC2Result<()> {
        if let Some(townhall) = self.units.my.townhalls.first() {
            // Setting rallypoint for command center
            townhall.smart(Target::Pos(self.start_center), false);

            // Ordering scv on initial 50 minerals
            townhall.train(UnitTypeId::Drone, false);
            self.subtract_resources(UnitTypeId::Drone, true);
        }

        self.get_worker_abilities();
        Ok(())
    }

    /// Called on every game step. (Main logic of the bot should be here)
    fn on_step(&mut self, _iteration: usize) -> SC2Result<()> {
        self.tactician();
        self.manage_workers();
        self.train_units();
        self.build();
        Ok(())
    }

    /// Called when different events happen.
    fn on_event(&mut self, event: Event) -> SC2Result<()> {
        match event {
            Event::UnitCreated(tag) => {
                if let Some(u) = self.units.my.units.get(tag) {
                    match u.type_id() {
                        drone if drone == self.race_values.start_townhall => {
                            println!("[Event][Construction Complete]\t{drone:?}");
                            let find_index = self.hatching.iter().position(|d| *d == drone);
                            if let Some(idx) = find_index {
                                self.hatching.remove(idx);
                            }
                        }
                        overlord if overlord == self.race_values.supply => {
                            println!("[Event][Unit Created]\t{overlord:?}");
                            let find_index = self.hatching.iter().position(|d| *d == overlord);
                            if let Some(idx) = find_index {
                                self.hatching.remove(idx);
                            }
                        }
                        unhandled => {
                            println!("[Event][Unit Created]\tUnhandled {unhandled:?}");
                            let find_index = self.hatching.iter().position(|d| *d == unhandled);
                            if let Some(idx) = find_index {
                                self.hatching.remove(idx);
                            }
                        }
                    }
                }
            }
            Event::ConstructionComplete(tag) => {
                if let Some(u) = self.units.my.structures.get(tag) {
                    match u.type_id() {
                        townhall if townhall == self.race_values.start_townhall => {
                            println!("[Event][Construction Complete]\t{townhall:?}")
                        }
                        unhandled => {
                            println!("[Event][Construction Complete]\tUnhandled {unhandled:?}")
                        }
                    }
                }
                if let Some(u) = self.units.my.structures.get(tag)
                    && u.type_id() == self.race_values.start_townhall
                    && let Some(idx) = self
                        .expansions
                        .iter()
                        .enumerate()
                        .find(|(_, exp)| exp.base == Some(tag))
                        .map(|(idx, _)| idx)
                {
                    self.base_indices.insert(tag, idx);
                }
            }
            Event::UnitDestroyed(tag, alliance) => {
                let remove_mineral = |bot: &mut SoyBot, tag| {
                    if let Some(ws) = bot.assigned.remove(&tag) {
                        for w in ws {
                            bot.harvesters.remove(&w);
                        }
                    }
                };

                match alliance {
                    Some(Alliance::Own) => {
                        // townhall destroyed
                        if let Some(idx) = self.base_indices.remove(&tag) {
                            let exp = &self.expansions[idx];
                            for m in exp.minerals.clone() {
                                remove_mineral(self, m);
                            }
                        // harvester died
                        } else if let Some((m, _)) = self.harvesters.remove(&tag) {
                            self.assigned.entry(m).and_modify(|ws| {
                                ws.remove(&tag);
                            });
                        // free worker died
                        } else {
                            println!("Worker died?");
                        }
                    }
                    // mineral mined out
                    Some(Alliance::Neutral) => remove_mineral(self, tag),
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Called once on last step with a result for your bot.
    fn on_end(&self, _result: GameResult) -> SC2Result<()> {
        Ok(())
    }
}
