use crate::soy_bot::SoyBot;
use rust_sc2::geometry::Point3;
use rust_sc2::prelude::*;

impl Player for SoyBot {
    fn get_player_settings(&'_ self) -> PlayerSettings<'_> {
        PlayerSettings::new(Race::Terran)
    }

    fn on_start(&mut self) -> SC2Result<()> {
        Ok(())
    }

    fn on_step(&mut self, _iteration: usize) -> SC2Result<()> {
        // Debug expansion locations
        for exp in self.expansions.clone() {
            let (loc, center) = (exp.loc, exp.center);
            let z = self.get_z_height(loc) + 1.5;
            self.debug
                .draw_sphere(loc.to3(z), 0.6, Some((255, 128, 255)));
            let z = self.get_z_height(center) + 1.5;
            self.debug
                .draw_sphere(center.to3(z), 0.5, Some((255, 128, 64)));
        }

        // Debug unit types
        self.units
            .all
            .iter()
            .map(|u| (format!("{:?}", u.type_id()), u.position3d()))
            .collect::<Vec<(String, Point3)>>()
            .into_iter()
            .for_each(|(s, pos): (String, Point3)| {
                self.debug
                    .draw_text_world(&s, pos, Some((255, 128, 128)), None)
            });
        self.assign_roles();
        self.execute_micro();
        Ok(())
    }

    fn on_end(&self, _result: GameResult) -> SC2Result<()> {
        Ok(())
    }

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
            Event::ConstructionStarted(_) => todo!(),
            Event::RandomRaceDetected(_race) => todo!(),
        }
        Ok(())
    }
}
