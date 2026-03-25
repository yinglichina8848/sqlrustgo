use clap::Parser;
use rand::Rng;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 1)]
    scale: u32,

    #[arg(long, default_value = ".")]
    output: String,
}

struct TpchDataGenerator {
    scale: u32,
    output_dir: PathBuf,
}

impl TpchDataGenerator {
    fn new(scale: u32, output: String) -> Self {
        Self {
            scale,
            output_dir: PathBuf::from(output),
        }
    }

    fn generate_all(&self) -> std::io::Result<()> {
        println!("Generating TPC-H data for SF={}", self.scale);

        std::fs::create_dir_all(&self.output_dir)?;

        let row_counts = self.get_row_counts();

        self.generate_customer(row_counts.customer)?;
        self.generate_orders(row_counts.orders)?;
        self.generate_lineitem(row_counts.lineitem)?;

        println!("Data generation complete!");
        println!("Files created in: {}", self.output_dir.display());

        Ok(())
    }

    fn get_row_counts(&self) -> RowCounts {
        let sf = self.scale as f64;
        RowCounts {
            customer: (1500.0 * sf) as usize,
            orders: (15000.0 * sf) as usize,
            lineitem: (60000.0 * sf) as usize,
        }
    }

    fn generate_customer(&self, count: usize) -> std::io::Result<()> {
        let mut file = File::create(self.output_dir.join("customer.csv"))?;
        writeln!(
            file,
            "c_custkey,c_name,c_address,c_nationkey,c_phone,c_acctbal,c_mktsegment,c_comment"
        )?;

        let mut rng = rand::thread_rng();
        let segments = [
            "AUTOMOBILE",
            "BUILDING",
            "FURNITURE",
            "MACHINERY",
            "HOUSEHOLD",
        ];

        for i in 1..=count {
            let custkey = i;
            let name = format!("Customer#{:09}", custkey);
            let address = format!(
                "{} {} {} {} {}",
                rng.gen::<u32>() % 100,
                self.random_string(10, &mut rng),
                self.random_string(4, &mut rng),
                self.random_number(9, &mut rng),
                self.random_number(6, &mut rng)
            );
            let nationkey = rng.gen::<u32>() % 25;
            let phone = format!(
                "{}-{}-{}",
                self.random_number(3, &mut rng),
                self.random_number(4, &mut rng),
                self.random_number(4, &mut rng)
            );
            let acctbal = (rng.gen::<f64>() * 9999.99 - 999.99).round() / 100.0;
            let mktsegment = segments[rng.gen::<usize>() % 5];
            let comment = self.random_string(29, &mut rng);

            writeln!(
                file,
                "{},{},{},{},{},{:.2},{},{}",
                custkey, name, address, nationkey, phone, acctbal, mktsegment, comment
            )?;
        }

        println!("  customer.csv: {} rows", count);
        Ok(())
    }

    fn generate_orders(&self, count: usize) -> std::io::Result<()> {
        let mut file = File::create(self.output_dir.join("orders.csv"))?;
        writeln!(file, "o_orderkey,o_custkey,o_orderstatus,o_totalprice,o_orderdate,o_orderpriority,o_clerk,o_shippriority,o_comment")?;

        let mut rng = rand::thread_rng();
        let statuses = ["O", "F", "P"];
        let priorities = ["1-URGENT", "2-HIGH", "3-MEDIUM", "4-NOT SPECIFIED", "5-LOW"];

        for i in 1..=count {
            let orderkey = i;
            let custkey = (rng.gen::<u32>() % 1500) + 1;
            let status = statuses[rng.gen::<usize>() % 3];
            let totalprice = (rng.gen::<f64>() * 99999.99).round() / 100.0;
            let orderdate = self.random_date(1992, 1998, &mut rng);
            let priority = priorities[rng.gen::<usize>() % 5];
            let clerk = format!("Clerk#{:06}", rng.gen::<u32>() % 1000);
            let shippriority = rng.gen::<u32>() % 100;
            let comment = self.random_string(19, &mut rng);

            writeln!(
                file,
                "{},{},{},{:.2},{},{},{},{},{}",
                orderkey,
                custkey,
                status,
                totalprice,
                orderdate,
                priority,
                clerk,
                shippriority,
                comment
            )?;
        }

        println!("  orders.csv: {} rows", count);
        Ok(())
    }

    fn generate_lineitem(&self, count: usize) -> std::io::Result<()> {
        let mut file = File::create(self.output_dir.join("lineitem.csv"))?;
        writeln!(file, "l_orderkey,l_partkey,l_suppkey,l_linenumber,l_quantity,l_extendedprice,l_discount,l_tax,l_returnflag,l_shipdate,l_commitdate,l_receiptdate,l_shipinstruct,l_shipmode,l_comment")?;

        let mut rng = rand::thread_rng();
        let return_flags = ["N", "R", "A"];
        let shipmodes = ["AIR", "AIR REG", "FOB", "MAIL", "RAIL", "SHIP", "TRUCK"];
        let instructs = [
            "DELIVER IN PERSON",
            "NONE",
            "TAKE BACK RETURN",
            "COLLECT COD",
        ];

        for i in 1..=count {
            let orderkey = ((i as f64 / 15.0).floor() as usize) + 1;
            let partkey = ((i as f64 / 4.0).floor() as usize) % 2000 + 1;
            let suppkey = ((i as f64 / 8.0).floor() as usize) % 1000 + 1;
            let linenumber = ((i - 1) % 7) + 1;
            let quantity = (rng.gen::<u32>() % 50) + 1;
            let extendedprice =
                (quantity as f64 * (rng.gen::<f64>() * 1000.0 + 100.0)).round() / 100.0;
            let discount = (rng.gen::<f64>() * 0.10).round() / 100.0;
            let tax = (rng.gen::<f64>() * 0.08).round() / 100.0;
            let returnflag = return_flags[rng.gen::<usize>() % 3];
            let shipdate = self.random_date(1992, 1998, &mut rng);
            let commitdate = self.random_date(1992, 1998, &mut rng);
            let receiptdate = self.random_date(1993, 1999, &mut rng);
            let instruct = instructs[rng.gen::<usize>() % 4];
            let shipmode = shipmodes[rng.gen::<usize>() % 7];
            let comment = self.random_string(14, &mut rng);

            writeln!(
                file,
                "{},{},{},{},{},{:.2},{:.2},{:.2},{},{},{},{},{},{},{}",
                orderkey,
                partkey,
                suppkey,
                linenumber,
                quantity,
                extendedprice,
                discount,
                tax,
                returnflag,
                shipdate,
                commitdate,
                receiptdate,
                instruct,
                shipmode,
                comment
            )?;
        }

        println!("  lineitem.csv: {} rows", count);
        Ok(())
    }

    fn random_string(&self, len: usize, rng: &mut impl Rng) -> String {
        const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        (0..len)
            .map(|_| {
                let idx = (rng.gen::<u32>() % CHARSET.len() as u32) as usize;
                CHARSET[idx] as char
            })
            .collect()
    }

    fn random_number(&self, len: usize, rng: &mut impl Rng) -> String {
        (0..len)
            .map(|_| char::from(b'0' + (rng.gen::<u32>() % 10) as u8))
            .collect()
    }

    fn random_date(&self, year_start: u32, year_end: u32, rng: &mut impl Rng) -> String {
        let year = year_start + rng.gen::<u32>() % (year_end - year_start + 1);
        let month = 1 + rng.gen::<u32>() % 12;
        let day = 1 + rng.gen::<u32>() % 28;
        format!("{:04}-{:02}-{:02}", year, month, day)
    }
}

struct RowCounts {
    customer: usize,
    orders: usize,
    lineitem: usize,
}

fn main() {
    let args = Args::parse();

    println!("TPC-H Data Generator");
    println!("====================");

    let generator = TpchDataGenerator::new(args.scale, args.output);

    if let Err(e) = generator.generate_all() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
