use napi_derive::napi;
use std::cell::RefCell;
use zavora_xlsx::ChartType;

#[napi]
pub struct Chart {
    pub(crate) inner: RefCell<zavora_xlsx::Chart>,
}

#[napi]
impl Chart {
    #[napi(constructor)]
    pub fn new(chart_type: String) -> napi::Result<Self> {
        let ct = match chart_type.as_str() {
            "bar" => ChartType::Bar,
            "column" => ChartType::Column,
            "line" => ChartType::Line,
            "pie" => ChartType::Pie,
            "scatter" => ChartType::Scatter,
            "area" => ChartType::Area,
            "doughnut" => ChartType::Doughnut,
            "radar" => ChartType::Radar,
            _ => {
                return Err(napi::Error::new(
                    napi::Status::InvalidArg,
                    format!(
                        "Unknown chart type '{}'. Valid types: bar, column, line, pie, scatter, area, doughnut, radar",
                        chart_type
                    ),
                ));
            }
        };
        Ok(Self {
            inner: RefCell::new(zavora_xlsx::Chart::new(ct)),
        })
    }

    #[napi]
    pub fn add_series(
        &self,
        values: String,
        categories: Option<String>,
        name: Option<String>,
    ) -> &Self {
        {
            let mut chart = self.inner.borrow_mut();
            let series = chart.add_series();
            series.set_values(&values);
            if let Some(ref cats) = categories {
                series.set_categories(cats);
            }
            if let Some(ref n) = name {
                series.set_name(n);
            }
        }
        self
    }

    #[napi]
    pub fn set_title(&self, title: String) -> &Self {
        {
            self.inner.borrow_mut().set_title(&title);
        }
        self
    }

    #[napi]
    pub fn set_x_axis_name(&self, name: String) -> &Self {
        {
            self.inner.borrow_mut().set_x_axis_name(&name);
        }
        self
    }

    #[napi]
    pub fn set_y_axis_name(&self, name: String) -> &Self {
        {
            self.inner.borrow_mut().set_y_axis_name(&name);
        }
        self
    }

    #[napi]
    pub fn set_size(&self, width: u32, height: u32) -> &Self {
        {
            let mut chart = self.inner.borrow_mut();
            chart.set_width(width);
            chart.set_height(height);
        }
        self
    }

    #[napi]
    pub fn set_legend_position(&self, position: String) -> &Self {
        let pos = match position.as_str() {
            "bottom" => zavora_xlsx::LegendPosition::Bottom,
            "top" => zavora_xlsx::LegendPosition::Top,
            "left" => zavora_xlsx::LegendPosition::Left,
            "right" => zavora_xlsx::LegendPosition::Right,
            "none" => zavora_xlsx::LegendPosition::None,
            _ => zavora_xlsx::LegendPosition::Bottom,
        };
        {
            self.inner.borrow_mut().set_legend_position(pos);
        }
        self
    }
}
