use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use zavora_xlsx::ChartType;

#[pyclass]
pub struct Chart {
    pub(crate) inner: zavora_xlsx::Chart,
}

#[pymethods]
impl Chart {
    #[new]
    pub fn new(chart_type: &str) -> PyResult<Self> {
        let ct = match chart_type {
            "bar" => ChartType::Bar,
            "column" => ChartType::Column,
            "line" => ChartType::Line,
            "pie" => ChartType::Pie,
            "scatter" => ChartType::Scatter,
            "area" => ChartType::Area,
            "doughnut" => ChartType::Doughnut,
            "radar" => ChartType::Radar,
            _ => {
                return Err(PyValueError::new_err(format!(
                    "Unknown chart type '{}'. Valid types: bar, column, line, pie, scatter, area, doughnut, radar",
                    chart_type
                )));
            }
        };
        Ok(Self {
            inner: zavora_xlsx::Chart::new(ct),
        })
    }

    #[pyo3(signature = (values, categories=None, name=None))]
    pub fn add_series<'a>(
        mut slf: PyRefMut<'a, Self>,
        values: &'a str,
        categories: Option<&'a str>,
        name: Option<&'a str>,
    ) -> PyRefMut<'a, Self> {
        {
            let series = slf.inner.add_series();
            series.set_values(values);
            if let Some(cats) = categories {
                series.set_categories(cats);
            }
            if let Some(n) = name {
                series.set_name(n);
            }
        }
        slf
    }

    pub fn set_title<'a>(mut slf: PyRefMut<'a, Self>, title: &'a str) -> PyRefMut<'a, Self> {
        slf.inner.set_title(title);
        slf
    }

    pub fn set_x_axis_name<'a>(mut slf: PyRefMut<'a, Self>, name: &'a str) -> PyRefMut<'a, Self> {
        slf.inner.set_x_axis_name(name);
        slf
    }

    pub fn set_y_axis_name<'a>(mut slf: PyRefMut<'a, Self>, name: &'a str) -> PyRefMut<'a, Self> {
        slf.inner.set_y_axis_name(name);
        slf
    }

    pub fn set_size(mut slf: PyRefMut<'_, Self>, width: u32, height: u32) -> PyRefMut<'_, Self> {
        slf.inner.set_width(width);
        slf.inner.set_height(height);
        slf
    }

    pub fn set_legend_position<'a>(
        mut slf: PyRefMut<'a, Self>,
        position: &'a str,
    ) -> PyRefMut<'a, Self> {
        let pos = match position {
            "bottom" => zavora_xlsx::LegendPosition::Bottom,
            "top" => zavora_xlsx::LegendPosition::Top,
            "left" => zavora_xlsx::LegendPosition::Left,
            "right" => zavora_xlsx::LegendPosition::Right,
            "none" => zavora_xlsx::LegendPosition::None,
            _ => zavora_xlsx::LegendPosition::Bottom,
        };
        slf.inner.set_legend_position(pos);
        slf
    }
}
