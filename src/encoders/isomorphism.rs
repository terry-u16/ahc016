use self::{
    annealing::{binarygraph::BinaryGraph, state::State},
    checker::{try_generate_isompic_graphs, IsomophicChecker, Vf2Checker},
};
use super::Encoder;
use crate::{
    encoders::isomorphism::annealing::annealer::Annealer,
    graph::Graph,
    utils::{decode_hex_to_u64, ChangeMinMax},
};
use rand_pcg::Pcg64Mcg;

mod annealing;
mod checker;

#[derive(Debug, Clone)]
pub struct IsomorphismEncoder {
    graphs: Vec<Graph>,
    /// 送信するグラフの種類数
    graph_count: usize,
    /// グラフの大きさ
    graph_size: usize,
    /// 冗長性考慮前のグラフの大きさ
    original_graph_size: usize,
    /// 冗長性
    redundancy: usize,
    /// 焼きなましスコアのグループ内:グループ外の重みの比
    score_coef: f64,
    /// 混同行列（使いやすいよう転置されている）
    confusing: Vec<Vec<u32>>,
}

impl IsomorphismEncoder {
    pub fn new(
        graph_count: usize,
        error_ratio: f64,
        bits: Option<usize>,
        redundancy: Option<usize>,
        score_coef: Option<f64>,
    ) -> Self {
        let (auto_bits, auto_redundancy, auto_score_coef) =
            Self::get_storategy(graph_count, error_ratio);

        let ((graphs, confusing), original_graph_size) = if let Some(bits) = bits {
            (
                try_generate_isompic_graphs(graph_count, error_ratio, bits).unwrap(),
                bits,
            )
        } else {
            (
                try_generate_isompic_graphs(graph_count, error_ratio, auto_bits).unwrap(),
                auto_bits,
            )
        };

        let redundancy = redundancy.unwrap_or(auto_redundancy);
        let score_coef = score_coef.unwrap_or(auto_score_coef);
        let graph_size = original_graph_size * redundancy;

        Self {
            graphs,
            graph_count,
            graph_size,
            original_graph_size,
            redundancy,
            score_coef,
            confusing,
        }
    }

    fn get_storategy(m: usize, error_ratio: f64) -> (usize, usize, f64) {
        let error_ratio = (error_ratio * 100.0 + 0.1) as usize;
        let storategy_matrix = get_storategy_matrix();
        storategy_matrix[m - 10][error_ratio]
    }

    fn restore(
        &self,
        graph: &BinaryGraph,
        annealer: &Annealer,
        duration: f64,
        rng: &mut Pcg64Mcg,
    ) -> Option<usize> {
        let state = State::init_rand(&graph, self.original_graph_size, self.score_coef, rng);
        let state = annealer.annealing(&graph, state, duration);
        let graph = state.restore_graph();
        let checker = Vf2Checker::new(&graph);

        for (i, g) in self.graphs.iter().enumerate() {
            if checker.is_isomorphic(g) {
                return Some(i);
            }
        }

        eprintln!("failed to decode.");
        None
    }
}

impl Encoder for IsomorphismEncoder {
    fn graph_size(&self) -> usize {
        self.graph_size
    }

    fn encode(&self, index: usize) -> Graph {
        let original_graph = &self.graphs[index];

        let mut graph = Graph::new(self.graph_size);
        // クリーク内
        for i in 0..original_graph.n {
            for x in 0..self.redundancy {
                for y in (x + 1)..self.redundancy {
                    let u = i * self.redundancy + x;
                    let v = i * self.redundancy + y;
                    graph.connect(u, v);
                }
            }
        }

        // クリーク間
        for i in 0..original_graph.n {
            for j in (i + 1)..original_graph.n {
                for x in 0..self.redundancy {
                    for y in 0..self.redundancy {
                        let u = i * self.redundancy + x;
                        let v = j * self.redundancy + y;
                        if original_graph[i][j] {
                            graph.connect(u, v);
                        }
                    }
                }
            }
        }

        graph
    }

    fn decode(&self, graph: &Graph, duration: f64) -> usize {
        let mut rng = Pcg64Mcg::new(42);
        let graph = BinaryGraph::new(graph);
        let annealer = Annealer::new(false);
        let mut votes = vec![0; self.graph_count];

        const TRIAL_COUNT: usize = 5;
        let each_duration = duration / TRIAL_COUNT as f64;

        // 多数決を取る
        for _ in 0..TRIAL_COUNT {
            if let Some(i) = self.restore(&graph, &annealer, each_duration, &mut rng) {
                for (j, &count) in self.confusing[i].iter().enumerate() {
                    votes[j] += count;
                }
            }
        }

        let mut max_votes = 0;
        let mut max_index = 0;

        for (i, &c) in votes.iter().enumerate() {
            if max_votes.change_max(c) {
                max_index = i;
            }
        }

        max_index
    }
}

fn get_storategy_matrix() -> Vec<Vec<(usize, usize, f64)>> {
    const HEX_DATA: &[u8] = b"40104010402a5020402f40304030403540354035404040404040404a4050405040504060406a407a406040705065408040804080408040ba40a040a040b040c040c040d0412550e0414541754175419551204010401040204020402f4020403040304030404040404040503a504040504050405f40604060505a4070407040854080408f409040904090409040a040b050b050ca50d0511a412551354175418541954140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502050205030503a503a5030603a504050405040504a504050555055505a50605060506050655070507f5080508050805090509050a050a050da50e550f55105511551355135514551105140501050205020502a502050305030503a503a60355040504f5040504a50455050505050555060605a506050605060607a5080508050905090509050a050c550ea50f5511a5115513551355135610051205140501050205020502a502050305030503a503a60355040504f5040504a50455050505050555060605a506050605060607a5080508050905090509050a050c550ea50f5511a5115513551355135610051205140501050205020502a502050305030503a503a60355040504f5040504a50455050505050555060605a506050605060607a5080508050905090509050a050c550ea50f5511a5115513551355135610051205140501050205020502a502050305030503a503a60355040504f5040504a50455050505050555060605a506050605060607a5080508050905090509050a050c550ea50f5511a5115513551355135610051205140501050205020502a502050305030503a503a60355040504f5040504a50455050505050555060605a506050605060607a5080508050905090509050a050c550ea50f5511a511551355135513561005120514050105020502a502f502a502a50305030503050305040504050405045504050405055505f50605065507560605070507050955080509f50af50bf50d550d550ea50fa60f560d051155135513561005120513050105020502a502f502a502a50305030503050305040504050405045504050405055505f50605065507560605070507050955080509f50af50bf50d550d550ea50fa60f560d051155135513561005120513050105020502a502f502a502a50305030503050305040504050405045504050405055505f50605065507560605070507050955080509f50af50bf50d550d550ea50fa60f560d051155135513561005120513050105020502050205030503050305030503a603a603a50405045504a50456045505a506050605060606a6060606050806075509a509f5090509050a050d550e560e55105610561056105514551105110512050105020502050205030503050305030503a603a603a50405045504a50456045505a506050605060606a6060606050806075509a509f5090509050a050d550e560e55105610561056105514551105110512050105010502a5020503050305030503a503f603a603a5040504560405050505a5060506050605060506050706060508050805080509050a550a550da60b050d560e560f560f561055125514551455110512050105020502050205020503a503a50305030603a50405040504050405040505550605060506f506050706060507050705090509a50906080609050c550d560b060d5610561056105512551355145512051405010502050205020602a503050305030603050306030504f6040604050505050505f506050605060507f507560655070607060806080508050b550b560c550da60d560e56105610a610551355145513051305010502050205020503050306030603a603a603a603a603f6045604a505050655060605a506050606065507f508a5070509a608050a560806090609060c560c560e560c061056105610560f051455120513060106020602a602a603560306030603a603a604060406040604060406050605060506055605a606560656065606a607060706080609a60a060a060bf60c560e560e56105610561056105610561006100610560106020602a602a603560306030603a603a604060406040604060406050605060506055605a606560656065606a607060706080609a60a060a060bf60c560e560e56105610561056105610561006100610560106020602a602a603560306030603a603a604060406040604060406050605060506055605a606560656065606a607060706080609a60a060a060bf60c560e560e56105610561056105610561006100610560106020602a602a603560306030603a603a604060406040604060406050605060506055605a606560656065606a607060706080609a60a060a060bf60c560e560e56105610561056105610561006100610560106020602a602a603560306030603a603a604060406040604060406050605060506055605a606560656065606a607060706080609a60a060a060bf60c560e560e56105610561056105610561006100610560106020602a602a603560306030603a603a604060406040604060406050605060506055605a606560656065606a607060706080609a60a060a060bf60c560e560e5610561056105610561056100610061056010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f56105610561006100610061006010602a602a602060306030603a603060306030603a60356040604560406050605f605f605f606a606060606060608060806080608060a560ba60b560b560d560d560fa60f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602a602a603a6030603a60306030603f6040604f604a6040604a6045605a605560506055605a6060606060856080608f609f60a5609060b560a060d560f560f560f561056105610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060256020602060306030603a604060406040604060406045605060506055605a605a6060606060706070608060806080609060b560b560b560d560fa60e56105610560f0610061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f061006100610060106020602060306030603f60306030603a603a604a604060406040604a605560606060606060656060606060606080608a60806090609060b560ca60c560d560fa60e56105610560f060f06100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a603060306040604060456045604060456050605060606060606560606070608060806080609060a560a060ba60d560e560f560f561056105610561006100610061006010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f061006100610061056010602060206020603060306030603a60306030603a60406040604060406040605f605a605f606a606060706070608560856080609060a560b560c560ca60d560f560f560f5610560f06100610061006105";

    let mut matrix = vec![];
    let mut cursor = 0;

    for _m in 10..=100 {
        let mut line = vec![];
        for _eps in 0..=40 {
            let bits = decode_hex_to_u64(&HEX_DATA[cursor..(cursor + 1)]) as usize;
            cursor += 1;
            let redundancy = decode_hex_to_u64(&HEX_DATA[cursor..(cursor + 2)]) as usize;
            cursor += 2;
            // (score_coef - 1) * 10 を格納している
            let score_coef = decode_hex_to_u64(&HEX_DATA[cursor..(cursor + 1)]) as f64 * 0.1 + 1.0;
            cursor += 1;
            line.push((bits, redundancy, score_coef));
        }
        matrix.push(line)
    }

    matrix
}
