use crate::soy_bot::SoyBot;
use rust_sc2::prelude::*;

impl Player for SoyBot {
    /// Returns settings used to connect bot to the game.
    fn get_player_settings(&'_ self) -> PlayerSettings<'_> {
        PlayerSettings::new(Race::Terran)
            .with_name("Soy Bot")
            .raw_crop_to_playable_area(true)
    }

    /// Called once on first step (i.e on game start).
    fn on_start(&mut self) -> SC2Result<()> {
        if let Some(townhall) = self.units.my.townhalls.first() {
            // Setting rallypoint for command center
            townhall.smart(Target::Pos(self.start_center), false);

            // Ordering scv on initial 50 minerals
            townhall.train(UnitTypeId::SCV, false);
            self.subtract_resources(UnitTypeId::SCV, true);
        }

        // Splitting workers to closest mineral crystals
        for u in &self.units.my.workers {
            if let Some(mineral) = self.units.mineral_fields.closest(u) {
                u.gather(mineral.tag(), false);
            }
        }

        Ok(())
    }

    /// Called on every game step. (Main logic of the bot should be here)
    fn on_step(&mut self, _iteration: usize) -> SC2Result<()> {
        self.assign_roles();
        self.execute_micro();
        Ok(())
    }

    /// Called when different events happen.
    fn on_event(&mut self, event: Event) -> SC2Result<()> {
        match event {
            Event::UnitCreated(tag) => {
                if let Some(u) = self.units.my.units.get(tag)
                    && u.type_id() == self.race_values.worker
                {
                    self.free_workers.insert(tag);
                }
            }
            Event::ConstructionComplete(tag) => {
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
                            bot.free_workers.insert(w);
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
                            self.free_workers.remove(&tag);
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
