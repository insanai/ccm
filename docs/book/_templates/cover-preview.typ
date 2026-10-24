#import "book.typ": book
#show: book.with(title: "CCM: Code, Change, and Confidence", author: "Vikrant Rathore",
  version: sys.inputs.at("version", default: "1.0.0-beta.1"), logo: "../../../assets/ccm-logo.svg")
