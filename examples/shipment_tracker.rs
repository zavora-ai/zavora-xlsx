use zavora_xlsx::*;
fn main() -> Result<()> {
    let mut wb = Workbook::new();
    let navy="#1B2A4A";let green="#0D7C3D";let red="#C00000";let blue="#2B579A";let amber="#E67E22";let border="#D6DCE4";let light="#F5F7FA";
    let ws=wb.worksheet(0)?;ws.set_name("Shipments")?;ws.hide_gridlines();
    let cw=[2.0,14.0,14.0,14.0,12.0,12.0,12.0,12.0,10.0,12.0,12.0];
    for(c,w)in cw.iter().enumerate(){ws.set_column_width(c as u16,*w)?;}
    let hdr=Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Center).border(BorderStyle::Thin);
    let hl=Format::new().bold().font_size(9.0).font_color("#FFFFFF").background_color(navy).align(Align::Left).border(BorderStyle::Thin);
    let tf=Format::new().font_size(10.0).font_color(navy).align(Align::Left).border(BorderStyle::Thin).border_color(border);
    let cf=Format::new().font_size(10.0).font_color(navy).align(Align::Center).border(BorderStyle::Thin).border_color(border);
    let df=Format::new().font_size(10.0).font_color(navy).align(Align::Center).num_format("m/d").border(BorderStyle::Thin).border_color(border);
    let mf=Format::new().font_size(10.0).font_color(navy).align(Align::Right).num_format("$#,##0").border(BorderStyle::Thin).border_color(border);
    let title=Format::new().bold().font_size(20.0).font_color(navy).align(Align::Left).align(Align::Bottom);
    let mut r=0u32;
    ws.set_row_height(r,6.0)?;r+=1;
    ws.write_with_format(r,1,"🚚 Shipment Tracker",&title)?;ws.set_row_height(r,32.0)?;r+=1;
    ws.write_with_format(r,1,"Logistics Dashboard — Q2 2025",&Format::new().font_size(10.0).font_color("#667085").italic())?;r+=1;
    for c in 1..=10u16{ws.write_with_format(r,c,"",&Format::new().background_color(blue))?;}ws.set_row_height(r,3.0)?;r+=2;
    let h=["Tracking #","Origin","Destination","Carrier","Ship Date","ETA","Status","Weight (kg)","Cost","Transit Days"];
    ws.write_with_format(r,1,h[0],&hdr)?;
    for(c,hh)in h[1..].iter().enumerate(){let f=if c<3{&hl}else{&hdr};ws.write_with_format(r,(c+2)as u16,*hh,f)?;}
    ws.set_row_height(r,22.0)?;r+=1;
    let shipments:Vec<(&str,&str,&str,&str,(i32,u32,u32),(i32,u32,u32),&str,f64,f64)>=vec![
        ("TRK-4001","Nairobi","Mombasa","KenFreight",(2025,3,28),(2025,3,30),"Delivered",450.0,1200.0),
        ("TRK-4002","Nairobi","Kisumu","SwiftLog",(2025,3,30),(2025,4,1),"Delivered",280.0,850.0),
        ("TRK-4003","Mombasa","Nairobi","KenFreight",(2025,4,1),(2025,4,3),"Delivered",620.0,1500.0),
        ("TRK-4004","Nairobi","Nakuru","ExpressKE",(2025,4,2),(2025,4,3),"Delivered",180.0,450.0),
        ("TRK-4005","Eldoret","Nairobi","SwiftLog",(2025,4,3),(2025,4,5),"In Transit",350.0,980.0),
        ("TRK-4006","Nairobi","Dar es Salaam","EALogistics",(2025,4,3),(2025,4,7),"In Transit",800.0,2800.0),
        ("TRK-4007","Mombasa","Kampala","EALogistics",(2025,4,4),(2025,4,8),"In Transit",520.0,2200.0),
        ("TRK-4008","Nairobi","Mombasa","KenFreight",(2025,4,4),(2025,4,6),"Delayed",380.0,1100.0),
        ("TRK-4009","Kisumu","Nairobi","ExpressKE",(2025,4,5),(2025,4,6),"Pending",200.0,600.0),
        ("TRK-4010","Nairobi","Kigali","EALogistics",(2025,4,5),(2025,4,9),"Pending",650.0,3200.0),
        ("TRK-4011","Thika","Nairobi","ExpressKE",(2025,4,1),(2025,4,1),"Delivered",90.0,250.0),
        ("TRK-4012","Nairobi","Malindi","SwiftLog",(2025,4,2),(2025,4,5),"Delayed",310.0,1400.0),
    ];
    let ds=r;
    for(i,s)in shipments.iter().enumerate(){
        let alt=i%2==1;
        let t=if alt{&Format::new().font_size(10.0).font_color(navy).align(Align::Left).background_color(light).border(BorderStyle::Thin).border_color(border)}else{&tf};
        ws.write_with_format(r,1,s.0,&cf)?;ws.write_with_format(r,2,s.1,t)?;ws.write_with_format(r,3,s.2,t)?;ws.write_with_format(r,4,s.3,t)?;
        ws.write_with_format(r,5,ExcelDateTime::from_ymd(s.4.0,s.4.1,s.4.2).unwrap(),&df)?;
        ws.write_with_format(r,6,ExcelDateTime::from_ymd(s.5.0,s.5.1,s.5.2).unwrap(),&df)?;
        let(sc,sb)=match s.6{"Delivered"=>(green,"#E8F5E9"),"In Transit"=>(blue,"#E3F2FD"),"Delayed"=>(red,"#FDE8E8"),_=>(amber,"#FFF8E1")};
        ws.write_with_format(r,7,s.6,&Format::new().font_size(10.0).font_color(sc).bold().align(Align::Center).background_color(sb).border(BorderStyle::Thin).border_color(border))?;
        ws.write_with_format(r,8,s.7,&cf)?;ws.write_with_format(r,9,s.8,&mf)?;
        ws.write_formula(r,10,&format!("IF(H{r1}=\"Delivered\",G{r1}-F{r1},MAX(0,TODAY()-F{r1}))",r1=r+1))?;ws.set_cell_format(r,10,&cf)?;
        r+=1;
    }
    let de=r-1;r+=1;
    let sl=Format::new().bold().font_size(10.0).font_color(navy).align(Align::Right);
    let sv=Format::new().bold().font_size(12.0).font_color(navy).align(Align::Center);
    ws.write_with_format(r,6,"Total Cost:",&sl)?;ws.write_formula(r,7,&format!("SUM(J{}:J{})",ds+1,de+1))?;
    ws.set_cell_format(r,7,&Format::new().bold().font_size(12.0).font_color(green).align(Align::Center).num_format("$#,##0"))?;
    ws.write_with_format(r,8,"Avg Transit:",&sl)?;ws.write_formula(r,9,&format!("AVERAGE(K{}:K{})",ds+1,de+1))?;
    ws.set_cell_format(r,9,&Format::new().bold().font_size(12.0).font_color(navy).align(Align::Center).num_format("0.0\" days\""))?;
    ws.add_conditional_format(ds,10,de,10,ConditionalFormatDataBar::new(blue))?;
    ws.set_autofilter(5,1,de,10);ws.set_freeze_panes(6,0)?;
    ws.set_landscape();ws.set_fit_to_page(1,0);ws.set_header("&CShipment Tracker");
    wb.save(home("shipment_tracker.xlsx"))?;println!("✅ Shipment Tracker saved");Ok(())
}
fn home(n:&str)->std::path::PathBuf{std::path::PathBuf::from(std::env::var("HOME").unwrap_or("/tmp".into())).join("Downloads").join(n)}
