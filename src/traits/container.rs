// The idea behind this trait is that, at least for now
// layout can be handled by dividing the window rectangle up into
// smaller rectangles
use rect::Rect;
use std::i32;

pub enum Side {
    Top,
    Bot,
    Lef,
    Rig,
}

fn percent_u32(whole_num: u32, percent: f64) -> Result<u32, &'static str> {
    if percent <= 100.0 && percent > 0.0 {
        let scaled_to_1e7 = percent * 100000.0;
        let divided = (((whole_num as f64) * scaled_to_1e7) / 10000000.0).floor();
        Ok(divided as u32)
    } else {
        Err("Invalid percent passed as an argument.")
    }
}


fn percent_i32(whole_num: u32, percent: f64) -> Result<i32, &'static str> {
    Ok(percent_u32(whole_num, percent)? as i32)
}

fn half_percent_i32(whole_num: u32, percent: f64) -> Result<i32, &'static str> {
    Ok(percent_i32(whole_num, percent)?/2)
}

fn vert_diff(top: Rect, bot: Rect) -> Result<i32, &'static str> {
    if top.height < (i32::MAX as u32) {
        Ok(((top.y + (top.height as i32)) - bot.y))
    } else {
        Err("Height too large to convert from u32 to i32")
    }
}

fn horz_diff(lef: Rect, rig: Rect) -> Result<i32, &'static str> {
    if lef.width < (i32::MAX as u32) {
        Ok(((lef.x + (lef.width as i32)) - rig.x))
    } else {
        Err("Width was too large to convert from u32 to i32")
    }
}


pub trait Container {
    fn center(&self, percent: f64) -> Result<Rect, &'static str>;
    fn shave(&self, percent: f64, tblr: Side) -> Result<Rect, &'static str>;
    fn split(&self, percent: f64, tblr: Side) -> Result<(Rect, Rect), &'static str>;
}

impl Container for Rect {
    // If passed a rectangle and a number less than 100, center will return a rectangle
    // that is percent size the original and centered in the original.
    // Returns an error if the percent is not in the range 0 < percent <= 100 
    fn center(&self, percent: f64) -> Result<Rect, &'static str> {
        let out_of_100 = 100.0 - percent;
        Ok(Rect::new((self.x + half_percent_i32(self.width, out_of_100)?),
                  (self.y + half_percent_i32(self.height, out_of_100)?),
                  (self.width - percent_u32(self.width, out_of_100)?),
                  (self.height - percent_u32(self.height, out_of_100)?)))
    }

    // If passed a Rect, a number less than 100, and a side, shave will return
    // a rectangle chopped down on that side to the percent given if
    // the percent is in the range 0 < percent <= 100. Otherwise, returns
    // an error.
    fn shave(&self, percent: f64, tblr: Side) -> Result<Rect, &'static str> {
        use self::Side::{Top, Bot, Lef, Rig};
        let out_of_100 = 100.0 - percent;
        match tblr {
            Top => { Ok(Rect::new(self.x,
                               (self.y + percent_i32(self.height, out_of_100)?),
                               self.width,
                               (self.height - percent_u32(self.height, out_of_100)?)))},
            Bot => { Ok(Rect::new(self.x, self.y, self.width,
                              (self.height - percent_u32(self.height, out_of_100)?)))},
            Lef => { Ok(Rect::new((self.x + percent_i32(self.width, out_of_100)?),
                               self.y,
                               (self.width - percent_u32(self.width, out_of_100)?),
                               self.height))},
            Rig => { Ok(Rect::new(self.x, self.y,
                                   (self.width - percent_u32(self.width, out_of_100)?),
                                   self.height))},
        }
    }

    // Splits the rect at the percent given
    // horizontally if Top or Bot is passed and vertically if Rig or Lef is passed.
    // Returns two touching Rects if the percent is valid, otherwise, returns
    // an error.
    // Resolves untouching Rects by making the pair taller or wider in sum.
     fn split(&self, percent:f64, tblr: Side) -> Result<(Rect, Rect), &'static str> {
         use self::Side::{Top, Bot, Lef, Rig};
         match tblr {
             Top | Bot => {let top_box: Rect = self.shave(percent, Bot)?;
                           let bot_box: Rect = self.shave((100.0 - percent), Top)?;
                           let vert_diff: i32 = vert_diff(top_box, bot_box)?;
                           match vert_diff {
                               1...i32::MAX => {Ok((top_box, Rect::new(bot_box.x,
                                                                       (bot_box.y + vert_diff),
                                                                       bot_box.width, bot_box.height)))}, // positive, the bottom of top passes bot, so add the diff to bot.y
                               0 => {Ok((top_box, bot_box))}, // no difference, _ should not be possble, use the same
                               _ => {let vert_diff_abs = vert_diff.abs() as u32;
                                               Ok((Rect::new(top_box.x, top_box.y, top_box.width,
                                                             (top_box.height + vert_diff_abs)), bot_box))}, // negative, bottom of top does not reach top of bot, so add (by abs value) the diff to top.height
                           } // Based on the rust playground, this way of comparing numbers is slow and makes a silly amount of assembler on debug but snappy and short on release
             },
             Lef | Rig => {let lef_box: Rect = self.shave(percent, Rig)?;
                           let rig_box = self.shave((100.0 - percent), Lef)?;
                           let horz_diff: i32 = horz_diff(lef_box, rig_box)?;
                           match horz_diff {
                               1...i32::MAX => {Ok((lef_box, Rect::new((rig_box.x + horz_diff),
                                                                       rig_box.y, rig_box.width, rig_box.height)))},
                               0 => {Ok((lef_box, rig_box))},
                               _ => {let horz_diff_abs = horz_diff.abs() as u32;
                                                 Ok((Rect::new(lef_box.x, lef_box.y,
                                                               (lef_box.width + horz_diff_abs), lef_box.height), rig_box))},
                           }
                               
             },
         }
     }
}
