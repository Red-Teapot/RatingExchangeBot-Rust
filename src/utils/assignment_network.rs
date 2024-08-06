use bimap::BiMap;
use serenity::all::UserId;
use std::collections::{HashMap, HashSet};

use crate::{
    models::{Exchange, PlayedGame, Submission, SubmissionId},
    solver::flow_network::{self, FlowNetwork},
};

#[derive(Debug)]
pub struct AssignmentNetwork {
    pub network: FlowNetwork,
    submissions: HashMap<SubmissionId, Submission>,
    submitter_nodes: BiMap<UserId, flow_network::Id>,
    submission_nodes: BiMap<SubmissionId, flow_network::Id>,
}

impl AssignmentNetwork {
    pub fn build(
        exchange: &Exchange,
        submissions: Vec<Submission>,
        played_games: &[PlayedGame],
    ) -> AssignmentNetwork {
        let submissions = {
            let mut map = HashMap::new();

            for submission in submissions {
                map.insert(submission.id, submission);
            }

            map
        };

        let submitter_played_games = {
            let mut map = HashMap::new();

            for played_game in played_games {
                let member = played_game.member;

                if !map.contains_key(&member) {
                    map.insert(member, HashSet::new());
                }

                map.get_mut(&member)
                    .expect("Checked separately")
                    .insert(played_game.link.clone());
            }

            map
        };

        let mut allocator = IndexAllocator::new();
        let source = allocator.next();
        let sink = allocator.next();
        let mut network = FlowNetwork::empty(source, sink);

        let (submitter_nodes, submission_nodes) = {
            let mut submitter_nodes = BiMap::new();
            let mut submission_nodes = BiMap::new();

            for submission in submissions.values() {
                let submitter_node = allocator.next();
                submitter_nodes.insert(submission.submitter, submitter_node);
                network.add_edge(
                    (source, submitter_node),
                    exchange.games_per_member.get() as _,
                    0,
                );

                let submission_node = allocator.next();
                submission_nodes.insert(submission.id, submission_node);
                network.add_edge(
                    (submission_node, sink),
                    exchange.games_per_member.get() as _,
                    0,
                );
            }

            (submitter_nodes, submission_nodes)
        };

        for src_submission in submissions.values() {
            let src_node = match submitter_nodes.get_by_left(&src_submission.submitter) {
                Some(node) => *node,
                None => continue,
            };

            let played_games = match submitter_played_games.get(&src_submission.submitter) {
                Some(games) => games,
                None => &HashSet::new(),
            };

            for dst_submission in submissions.values() {
                if src_submission.submitter == dst_submission.submitter {
                    continue;
                }
                if played_games.contains(&dst_submission.link) {
                    continue;
                }

                let dst_node = match submission_nodes.get_by_left(&dst_submission.id) {
                    Some(node) => *node,
                    None => continue,
                };

                network.add_edge((src_node, dst_node), 1, 0);
            }
        }

        AssignmentNetwork {
            network,
            submissions,
            submitter_nodes,
            submission_nodes,
        }
    }

    pub fn get_assignments(&self) -> HashMap<UserId, Vec<Submission>> {
        let mut map = HashMap::new();

        for (&user_id, &node) in &self.submitter_nodes {
            // TODO: Maybe log something if an entry was not found
            let assignments = self
                .network
                .outgoing_edges(node)
                .iter()
                .filter(|&&edge| self.network.flow(edge) > 0)
                .filter_map(|edge| self.submission_nodes.get_by_right(&edge.end))
                .filter_map(|submission_id| self.submissions.get(submission_id))
                .map(|submission| submission.clone())
                .collect();

            map.insert(user_id, assignments);
        }

        map
    }
}

struct IndexAllocator {
    index: u16,
}

impl IndexAllocator {
    pub fn new() -> IndexAllocator {
        IndexAllocator { index: 0 }
    }

    pub fn next(&mut self) -> u16 {
        let result = self.index;
        self.index += 1;
        result
    }
}

#[cfg(test)]
mod test {
    use std::num::NonZero;

    use bimap::BiHashMap;
    use map_macro::hash_map;
    use serenity::all::{ChannelId, GuildId, UserId};
    use time::macros::datetime;

    use crate::{
        jam_types::JamType,
        models::{
            types::UtcDateTime, Exchange, ExchangeId, ExchangeState, PlayedGame, PlayedGameId,
            Submission, SubmissionId,
        },
        solver::flow_network::{edge, FlowNetwork},
    };

    use super::AssignmentNetwork;

    #[test]
    fn empty() {
        let exchange = Exchange {
            id: ExchangeId(1),
            guild: GuildId::new(2),
            channel: ChannelId::new(3),
            jam_type: JamType::Itch,
            jam_link: "https://itch.io/jam/example-jam".to_string(),
            slug: "Test".to_string(),
            display_name: "Test".to_string(),
            state: ExchangeState::AcceptingSubmissions,
            submissions_start: UtcDateTime::assume_utc(datetime!(2024-01-01 12:00:00)),
            submissions_end: UtcDateTime::assume_utc(datetime!(2024-01-01 13:00:00)),
            games_per_member: NonZero::new(5).unwrap(),
        };
        let submissions = vec![];
        let played_games = vec![];

        let network = AssignmentNetwork::build(&exchange, submissions, &played_games);

        network.network.validate(Some(0)).unwrap();
        assert!(network.network.edges().is_empty());
        assert!(network.submitter_nodes.is_empty());
        assert!(network.submission_nodes.is_empty());
    }

    #[test]
    fn one_submitter() {
        let exchange = Exchange {
            id: ExchangeId(1),
            guild: GuildId::new(2),
            channel: ChannelId::new(3),
            jam_type: JamType::Itch,
            jam_link: "https://itch.io/jam/example-jam".to_string(),
            slug: "Test".to_string(),
            display_name: "Test".to_string(),
            state: ExchangeState::AcceptingSubmissions,
            submissions_start: UtcDateTime::assume_utc(datetime!(2024-01-01 12:00:00)),
            submissions_end: UtcDateTime::assume_utc(datetime!(2024-01-01 13:00:00)),
            games_per_member: NonZero::new(5).unwrap(),
        };
        let submissions = vec![Submission {
            id: SubmissionId(1),
            exchange_id: exchange.id,
            link: "https://itch.io/jam/example-jam/rate/123456".to_string(),
            submitter: UserId::new(1),
            submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
        }];
        let played_games = vec![];

        let network = AssignmentNetwork::build(&exchange, submissions, &played_games);

        network.network.validate(Some(0)).unwrap();
        assert_eq!(network.submitter_nodes.len(), 1);
        assert!(network.submitter_nodes.contains_left(&UserId::new(1)));
        assert_eq!(network.submission_nodes.len(), 1);
        assert!(network.submission_nodes.contains_left(&SubmissionId(1)));
        assert_eq!(network.network.edges().len(), 2);
        assert!(network.network.edges().contains(&edge(
            network.network.source(),
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(1))
                .unwrap()
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(1))
                .unwrap(),
            network.network.sink(),
        )));
    }

    #[test]
    fn multiple_submitters() {
        let exchange = Exchange {
            id: ExchangeId(1),
            guild: GuildId::new(2),
            channel: ChannelId::new(3),
            jam_type: JamType::Itch,
            jam_link: "https://itch.io/jam/example-jam".to_string(),
            slug: "Test".to_string(),
            display_name: "Test".to_string(),
            state: ExchangeState::AcceptingSubmissions,
            submissions_start: UtcDateTime::assume_utc(datetime!(2024-01-01 12:00:00)),
            submissions_end: UtcDateTime::assume_utc(datetime!(2024-01-01 13:00:00)),
            games_per_member: NonZero::new(3).unwrap(),
        };
        let submissions = vec![
            Submission {
                id: SubmissionId(1),
                exchange_id: exchange.id,
                link: "https://itch.io/jam/example-jam/rate/000001".to_string(),
                submitter: UserId::new(1),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            },
            Submission {
                id: SubmissionId(2),
                exchange_id: exchange.id,
                link: "https://itch.io/jam/example-jam/rate/000002".to_string(),
                submitter: UserId::new(2),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            },
            Submission {
                id: SubmissionId(3),
                exchange_id: exchange.id,
                link: "https://itch.io/jam/example-jam/rate/000003".to_string(),
                submitter: UserId::new(3),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            },
            Submission {
                id: SubmissionId(4),
                exchange_id: exchange.id,
                link: "https://itch.io/jam/example-jam/rate/000004".to_string(),
                submitter: UserId::new(4),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            },
        ];
        let played_games = vec![
            PlayedGame {
                id: PlayedGameId(24),
                link: "https://itch.io/jam/example-jam/rate/000004".to_string(),
                member: UserId::new(2),
                is_manual: false,
            },
            PlayedGame {
                id: PlayedGameId(31),
                link: "https://itch.io/jam/example-jam/rate/000001".to_string(),
                member: UserId::new(3),
                is_manual: false,
            },
            PlayedGame {
                id: PlayedGameId(34),
                link: "https://itch.io/jam/example-jam/rate/000004".to_string(),
                member: UserId::new(3),
                is_manual: false,
            },
            PlayedGame {
                id: PlayedGameId(41),
                link: "https://itch.io/jam/example-jam/rate/000001".to_string(),
                member: UserId::new(4),
                is_manual: false,
            },
            PlayedGame {
                id: PlayedGameId(42),
                link: "https://itch.io/jam/example-jam/rate/000002".to_string(),
                member: UserId::new(4),
                is_manual: false,
            },
            PlayedGame {
                id: PlayedGameId(43),
                link: "https://itch.io/jam/example-jam/rate/000003".to_string(),
                member: UserId::new(4),
                is_manual: false,
            },
        ];

        let network = AssignmentNetwork::build(&exchange, submissions, &played_games);

        network.network.validate(Some(0)).unwrap();
        assert_eq!(network.submitter_nodes.len(), 4);
        assert!(network.submitter_nodes.contains_left(&UserId::new(1)));
        assert!(network.submitter_nodes.contains_left(&UserId::new(2)));
        assert!(network.submitter_nodes.contains_left(&UserId::new(3)));
        assert!(network.submitter_nodes.contains_left(&UserId::new(4)));
        assert_eq!(network.submission_nodes.len(), 4);
        assert!(network.submission_nodes.contains_left(&SubmissionId(1)));
        assert!(network.submission_nodes.contains_left(&SubmissionId(2)));
        assert!(network.submission_nodes.contains_left(&SubmissionId(3)));
        assert!(network.submission_nodes.contains_left(&SubmissionId(4)));
        assert_eq!(network.network.edges().len(), 14);
        assert!(network.network.edges().contains(&edge(
            network.network.source(),
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(1))
                .unwrap()
        )));
        assert!(network.network.edges().contains(&edge(
            network.network.source(),
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(2))
                .unwrap()
        )));
        assert!(network.network.edges().contains(&edge(
            network.network.source(),
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(3))
                .unwrap()
        )));
        assert!(network.network.edges().contains(&edge(
            network.network.source(),
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(4))
                .unwrap()
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(1))
                .unwrap(),
            network.network.sink(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(2))
                .unwrap(),
            network.network.sink(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(3))
                .unwrap(),
            network.network.sink(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(4))
                .unwrap(),
            network.network.sink(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(1))
                .unwrap(),
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(2))
                .unwrap(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(1))
                .unwrap(),
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(3))
                .unwrap(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(1))
                .unwrap(),
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(4))
                .unwrap(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(2))
                .unwrap(),
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(1))
                .unwrap(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(2))
                .unwrap(),
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(3))
                .unwrap(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(1))
                .unwrap(),
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(3))
                .unwrap(),
        )));
        assert!(network.network.edges().contains(&edge(
            *network
                .submitter_nodes
                .get_by_left(&UserId::new(3))
                .unwrap(),
            *network
                .submission_nodes
                .get_by_left(&SubmissionId(2))
                .unwrap(),
        )));
    }

    #[test]
    fn getting_assignments() {
        let network = AssignmentNetwork {
            network: {
                let source = 0;
                let sink = 1;
                let submitters = vec![2, 3, 4, 5];
                let submissions = vec![6, 7, 8, 9];

                let mut net = FlowNetwork::empty(source, sink);

                net.add_edge((source, submitters[0]), 5, 3);
                net.add_edge((source, submitters[1]), 5, 2);
                net.add_edge((source, submitters[2]), 5, 1);
                net.add_edge((source, submitters[3]), 5, 0);

                net.add_edge((submitters[0], submissions[1]), 1, 1);
                net.add_edge((submitters[0], submissions[2]), 1, 0);
                net.add_edge((submitters[0], submissions[3]), 1, 1);
                net.add_edge((submitters[1], submissions[0]), 1, 1);
                net.add_edge((submitters[1], submissions[2]), 1, 1);
                net.add_edge((submitters[2], submissions[1]), 1, 1);

                net.add_edge((submissions[0], sink), 5, 1);
                net.add_edge((submissions[1], sink), 5, 2);
                net.add_edge((submissions[2], sink), 5, 1);
                net.add_edge((submissions[3], sink), 5, 1);

                net
            },
            submissions: hash_map! {
                SubmissionId(1) => Submission {
                    id: SubmissionId(1),
                    exchange_id: ExchangeId(1),
                    link: "https://itch.io/example-jam/rate/000001".to_string(),
                    submitter: UserId::new(1),
                    submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
                },
                SubmissionId(2) => Submission {
                    id: SubmissionId(2),
                    exchange_id: ExchangeId(1),
                    link: "https://itch.io/example-jam/rate/000002".to_string(),
                    submitter: UserId::new(2),
                    submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
                },
                SubmissionId(3) => Submission {
                    id: SubmissionId(3),
                    exchange_id: ExchangeId(1),
                    link: "https://itch.io/example-jam/rate/000003".to_string(),
                    submitter: UserId::new(3),
                    submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
                },
                SubmissionId(4) => Submission {
                    id: SubmissionId(4),
                    exchange_id: ExchangeId(1),
                    link: "https://itch.io/example-jam/rate/000004".to_string(),
                    submitter: UserId::new(4),
                    submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
                },
            },
            submitter_nodes: {
                let mut map = BiHashMap::new();

                map.insert(UserId::new(1), 2);
                map.insert(UserId::new(2), 3);
                map.insert(UserId::new(3), 4);
                map.insert(UserId::new(4), 5);

                map
            },
            submission_nodes: {
                let mut map = BiHashMap::new();

                map.insert(SubmissionId(1), 6);
                map.insert(SubmissionId(2), 7);
                map.insert(SubmissionId(3), 8);
                map.insert(SubmissionId(4), 9);

                map
            },
        };

        let assignments = network.get_assignments();

        {
            let assignments = assignments.get(&UserId::new(1)).unwrap();
            assert_eq!(assignments.len(), 2);
            assert!(assignments.contains(&Submission {
                id: SubmissionId(2),
                exchange_id: ExchangeId(1),
                link: "https://itch.io/example-jam/rate/000002".to_string(),
                submitter: UserId::new(2),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            }));
            assert!(assignments.contains(&Submission {
                id: SubmissionId(4),
                exchange_id: ExchangeId(1),
                link: "https://itch.io/example-jam/rate/000004".to_string(),
                submitter: UserId::new(4),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            }));
        }
        {
            let assignments = assignments.get(&UserId::new(2)).unwrap();
            assert_eq!(assignments.len(), 2);
            assert!(assignments.contains(&Submission {
                id: SubmissionId(1),
                exchange_id: ExchangeId(1),
                link: "https://itch.io/example-jam/rate/000001".to_string(),
                submitter: UserId::new(1),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            }));
            assert!(assignments.contains(&Submission {
                id: SubmissionId(3),
                exchange_id: ExchangeId(1),
                link: "https://itch.io/example-jam/rate/000003".to_string(),
                submitter: UserId::new(3),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            }));
        }
        {
            let assignments = assignments.get(&UserId::new(3)).unwrap();
            assert_eq!(assignments.len(), 1);
            assert!(assignments.contains(&Submission {
                id: SubmissionId(2),
                exchange_id: ExchangeId(1),
                link: "https://itch.io/example-jam/rate/000002".to_string(),
                submitter: UserId::new(2),
                submitted_at: UtcDateTime::assume_utc(datetime!(2020-01-01 00:00:00)),
            }));
        }
        {
            let assignments = assignments.get(&UserId::new(4)).unwrap();
            assert_eq!(assignments.len(), 0);
        }
    }
}
