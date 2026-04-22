use std::sync::{Arc, Mutex};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::error::IntoNapi;
use crate::worksheet::Worksheet;

#[napi]
pub struct Workbook {
    pub(crate) inner: Arc<Mutex<zavora_xlsx::Workbook>>,
}

#[napi(object)]
pub struct DocPropertiesJs {
    pub title: Option<String>,
    pub author: Option<String>,
    pub subject: Option<String>,
    pub description: Option<String>,
    pub keywords: Option<String>,
    pub category: Option<String>,
    pub company: Option<String>,
}

#[napi]
impl Workbook {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(zavora_xlsx::Workbook::new())),
        }
    }

    #[napi(factory)]
    pub fn open(path: String) -> napi::Result<Self> {
        let wb = zavora_xlsx::Workbook::open(&path).into_napi()?;
        Ok(Self {
            inner: Arc::new(Mutex::new(wb)),
        })
    }

    #[napi(factory)]
    pub fn open_from_buffer(buffer: Buffer) -> napi::Result<Self> {
        let wb = zavora_xlsx::Workbook::open_from_buffer(&buffer).into_napi()?;
        Ok(Self {
            inner: Arc::new(Mutex::new(wb)),
        })
    }

    #[napi]
    pub fn save(&self, path: String) -> napi::Result<()> {
        let mut wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        wb.save(&path).into_napi()
    }

    #[napi]
    pub fn save_to_buffer(&self) -> napi::Result<Buffer> {
        let mut wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        let bytes = wb.save_to_buffer().into_napi()?;
        Ok(Buffer::from(bytes))
    }

    #[napi]
    pub fn worksheet(&self, index: u32) -> napi::Result<Worksheet> {
        Ok(Worksheet {
            workbook: Arc::clone(&self.inner),
            index: index as usize,
        })
    }

    #[napi]
    pub fn add_worksheet(&self) -> napi::Result<Worksheet> {
        let mut wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        wb.add_worksheet();
        let idx = wb.sheet_count() - 1;
        drop(wb);
        Ok(Worksheet {
            workbook: Arc::clone(&self.inner),
            index: idx,
        })
    }

    #[napi]
    pub fn add_worksheet_with_name(&self, name: String) -> napi::Result<Worksheet> {
        let mut wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        wb.add_worksheet_with_name(&name).into_napi()?;
        let idx = wb.sheet_count() - 1;
        drop(wb);
        Ok(Worksheet {
            workbook: Arc::clone(&self.inner),
            index: idx,
        })
    }

    #[napi]
    pub fn sheet_names(&self) -> napi::Result<Vec<String>> {
        let wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        Ok(wb.sheet_names().iter().map(|s| s.to_string()).collect())
    }

    #[napi]
    pub fn sheet_count(&self) -> napi::Result<u32> {
        let wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        Ok(wb.sheet_count() as u32)
    }

    #[napi]
    pub fn set_properties(&self, props: DocPropertiesJs) -> napi::Result<()> {
        let mut wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        let mut dp = zavora_xlsx::DocProperties::new();
        if let Some(ref t) = props.title { dp = dp.title(t); }
        if let Some(ref a) = props.author { dp = dp.author(a); }
        if let Some(ref s) = props.subject { dp = dp.subject(s); }
        if let Some(ref d) = props.description { dp = dp.description(d); }
        if let Some(ref k) = props.keywords { dp = dp.keywords(k); }
        if let Some(ref c) = props.category { dp = dp.category(c); }
        if let Some(ref c) = props.company { dp = dp.company(c); }
        wb.set_properties(dp);
        Ok(())
    }

    #[napi]
    pub fn properties(&self) -> napi::Result<DocPropertiesJs> {
        let wb = self.inner.lock().map_err(|_| {
            napi::Error::new(napi::Status::GenericFailure, "lock poisoned")
        })?;
        let p = wb.properties();
        Ok(DocPropertiesJs {
            title: p.title.clone(),
            author: p.author.clone(),
            subject: p.subject.clone(),
            description: p.description.clone(),
            keywords: p.keywords.clone(),
            category: p.category.clone(),
            company: p.company.clone(),
        })
    }
}
