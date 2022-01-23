use std::{fmt::Debug, str::FromStr};

pub fn is_path_variable(st: &str) -> bool {
    st.starts_with(':')
}

pub struct PathVariable {
    it: Box<dyn Iterator<Item = String> + Send>,
}

impl PathVariable {
    pub fn new(request_path: &str, defined_path: &str) -> Self {
        let variable_indexes = defined_path
            .split('/')
            .enumerate()
            .filter(|(_, e)| is_path_variable(e))
            .map(|(i, _)| i);

        let mut splitted_request_path = request_path.split('/').enumerate();

        let path_variables = variable_indexes
            .map(|i| {
                splitted_request_path
                    .find(|(j, _)| &i == j)
                    .map(|(_, var)| var.into())
                    .unwrap()
            })
            .collect::<Vec<_>>();

        path_variables.into()
    }

    pub fn next_variable<T>(&mut self) -> Option<T>
    where
        T: FromStr,
        T::Err: Debug,
    {
        self.it.next().and_then(|var| var.parse::<T>().ok())
    }
}

impl From<Vec<String>> for PathVariable {
    fn from(variables: Vec<String>) -> Self {
        Self {
            it: Box::new(variables.into_iter()),
        }
    }
}

impl From<(&str, &str)> for PathVariable {
    fn from((req_path, pattern): (&str, &str)) -> Self {
        PathVariable::new(req_path, pattern)
    }
}
